#!/usr/bin/env python3

import os
import sys
import subprocess
import re

TARGETS = [
    'tiny-skia',
    'skia',
    'cairo',
    'raqote',
]

COLORS = [
    '#fc7f37',
    '#d8317f',
    '#fcc100',
    '#3131d8',
]

BLEND_ORDER = [
    ['clear', 'Clear'],
    ['source', 'Source'],
    ['destination', 'Destination'],
    ['source_over', 'Source Over'],
    ['destination_over', 'Destination Over'],
    ['source_in', 'Source In'],
    ['destination_in', 'Destination In'],
    ['source_out', 'Source Out'],
    ['destination_out', 'Destination Out'],
    ['source_atop', 'Source Atop'],
    ['destination_atop', 'Destination Atop'],
    ['xor', 'Xor'],
    ['plus', 'Plus'],
    ['modulate', 'Modulate'],
    ['screen', 'Screen'],
    ['overlay', 'Overlay'],
    ['darken', 'Darken'],
    ['lighten', 'Lighten'],
    ['color_dodge', 'Color Dodge'],
    ['color_burn', 'Color Burn'],
    ['hard_light', 'Hard Light'],
    ['soft_light', 'Soft Light'],
    ['difference', 'Difference'],
    ['exclusion', 'Exclusion'],
    ['multiply', 'Multiply'],
    ['hue', 'Hue'],
    ['saturation', 'Saturation'],
    ['color', 'Color'],
    ['luminosity', 'Luminosity'],
]

GRADIENT_ORDER = [
    ['two_stops_linear_pad', '[\'Linear\', \'two stops\', \'pad\']'],
    ['two_stops_linear_reflect', '[\'Linear\', \'two stops\', \'reflect\']'],
    ['two_stops_linear_repeat', '[\'Linear\', \'two stops\', \'repeat\']'],
    ['three_stops_linear_even', '[\'Linear\', \'three stops\', \'evenly spread\']'],
    ['three_stops_linear_uneven', '[\'Linear\', \'three stops\', \'unevenly spread\']'],
    ['simple_radial', 'Simple radial'],
    ['two_point_radial','Two point radial'],
]

PATTERN_ORDER = [
    ['plain', 'Nearest'],
    ['lq', 'Bilinear'],
    ['hq', 'Bicubic/Gauss'],
]

CLIP_ORDER = [
    ['clip', 'No AA'],
    ['aa', 'AA'],
]

FILL_ORDER = [
    ['all', 'All'],
    ['rect', 'Rect'],
    ['rect_aa', 'Rect AA'],
    ['rect_aa_ts', 'Rect AA transformed'],
    ['path_aa', 'Path AA'],
    ['source', 'Source'],
    ['opaque', 'Opaque'],
]

HAIRLINE_ORDER = [
    ['hairline', 'No AA'],
    ['aa', 'AA'],
]

SPIRAL_ORDER = [
    ['spiral', 'Spiral'],
]


def parse_output(output, results):
    for line in output.split('\n'):
        if not line.startswith('test'):
            continue

        if 'test result' in line:
            continue

        name = re.sub(r'test +', '', line)
        name = re.sub(r' .*', '', name)

        bench, name = name.split('::')

        if name.endswith('tiny_skia'):
            name = name[:-10]
            lib = 'tiny-skia'
        elif name.endswith('skia'):
            name = name[:-5]
            lib = 'skia'
        elif name.endswith('raqote'):
            name = name[:-7]
            lib = 'raqote'
        elif name.endswith('cairo'):
            name = name[:-6]
            lib = 'cairo'

        if not name:
            name = bench

        time = re.sub(r'.*bench: +', '', line)
        time = re.sub(r' ns.*', '', time)
        time = time.replace(',', '')

        results.append([lib, bench, name, time])


def collect_values(bench_name, target, order, results):
    values = {}
    for r in results:
        lib, bench, name, time = r
        if lib == target and bench == bench_name:
            values[name] = time

    sorted_values = []
    for v, _ in order:
        if v in values:
            sorted_values.append(values[v])
        else:
            sorted_values.append('0')

    return sorted_values


def generate_chart_js(name, title, order, results):
    chart_data = f'      const {name}Labels = [\n'
    for _, l in order:
        if '[' in l:
            chart_data += f'        {l},\n'
        else:
            chart_data += f'        \'{l}\',\n'

    chart_data += f'      ]\n\n'

    chart_data += f'      const {name}Data = {{\n'
    chart_data += f'        labels: {name}Labels,\n'
    chart_data += f'        datasets: [\n'

    for target, color in zip(TARGETS, COLORS):
        chart_data += '          {\n'
        chart_data += f'            label: \'{target}\',\n'
        chart_data += f'            backgroundColor: \'{color}\',\n'
        chart_data += f'            borderColor: \'{color}\',\n'
        values_str = ', '.join(collect_values(name, target, order, results))
        chart_data += f'            data: [{values_str}],\n'
        chart_data += '          },\n'

    chart_data += '        ]\n'
    chart_data += '      };\n'

    chart_data += '''
      const NAME_PLACEHOLDERConfig = {
        type: 'bar',
        data: NAME_PLACEHOLDERData,
        options: {
          responsive: true,
          maintainAspectRatio: false,
          indexAxis: 'y',
          elements: {
            bar: {
              borderWidth: 2,
            }
          },
          plugins: {
            legend: {
              position: 'right',
            },
            title: {
              display: true,
              text: 'TITLE_PLACEHOLDER'
            },
          },
          scales: {
            x: {
              ticks: {
                callback: function(val, index) {
                  return this.getLabelForValue(val) + 'ns';
                },
              }
            }
          }
        },
      };

      new Chart(
        document.getElementById('NAME_PLACEHOLDERChart'),
        NAME_PLACEHOLDERConfig
      );\n\n'''.replace('TITLE_PLACEHOLDER', title).replace('NAME_PLACEHOLDER', name)

    return chart_data


if len(sys.argv) != 2:
    print(
        'Error: Skia dir is not set.\n'
        '\n'
        'Use: gen-charts-arm64.py /path/to/skia/dir\n'
        '\n'
        'The Skia dir should have the following structure:\n'
        '- include (directory copy-pasted from Skia sources)\n'
        '- libskia.so (skia lib built with -march=native)'
    )
    exit(1)

skia_dir = sys.argv[1]

# collect benchmarks output first
results = []

os.environ['SKIA_DIR'] = skia_dir
os.environ['SKIA_LIB_DIR'] = skia_dir
os.environ['LD_LIBRARY_PATH'] = skia_dir
os.environ['RUSTFLAGS'] = '-Ctarget-cpu=native'

output = subprocess.run(
    ['rustup', 'run', 'nightly', 'cargo', 'bench', '--all-features'],
    check=True, capture_output=True).stdout.decode()

parse_output(output, results)

header = '''
<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8"/>
    <title>tiny-skia benchmarks</title>
  </head>
  <body>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <div style="height: 1500px">
      <canvas id="blendChart"></canvas>
    </div>
    <div style="height: 500px">
      <canvas id="fillChart"></canvas>
    </div>
    <div style="height: 500px">
      <canvas id="gradientsChart"></canvas>
    </div>
    <div style="height: 200px">
      <canvas id="patternsChart"></canvas>
    </div>
    <div style="height: 170px">
      <canvas id="clipChart"></canvas>
    </div>
    <div style="height: 170px">
      <canvas id="hairlineChart"></canvas>
    </div>
    <div style="height: 110px">
      <canvas id="spiralChart"></canvas>
    </div>
    <script>
'''

footer = '''
    </script>
  </body>
</html>
'''

with open('arm64.html', 'w') as f:
    f.write(header)
    f.write(generate_chart_js('blend', 'Blending modes', BLEND_ORDER, results))
    f.write(generate_chart_js('fill', 'Fill', FILL_ORDER, results))
    f.write(generate_chart_js('gradients', 'Gradients', GRADIENT_ORDER, results))
    f.write(generate_chart_js('patterns', 'Patterns', PATTERN_ORDER, results))
    f.write(generate_chart_js('clip', 'Clip', CLIP_ORDER, results))
    f.write(generate_chart_js('hairline', 'Hairline stroking', HAIRLINE_ORDER, results))
    f.write(generate_chart_js('spiral', 'Stroke spiral', SPIRAL_ORDER, results))
    f.write(footer)
