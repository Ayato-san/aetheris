#!/usr/bin/env python3
"""Convert SVG fill paths from a file or folder into Aetheris GPUI icon components."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path


NUMBER = r"[-+]?(?:\d+\.?(?:\d*)?|\.\d+)(?:[eE][-+]?\d+)?"
TOKEN_RE = re.compile(rf"[AaCcHhLlMmQqSsTtVvZz]|{NUMBER}")
SEPARATOR_RE = re.compile(r"^[\s,]*$")
PARAMETERS = {"M": 2, "L": 2, "H": 1, "V": 1, "C": 6, "S": 4, "Q": 4, "T": 2, "A": 7}


class ConversionError(ValueError):
    pass


@dataclass(frozen=True)
class Operation:
    name: str
    values: tuple[float | bool, ...] = ()


@dataclass(frozen=True)
class SvgPath:
    operations: tuple[Operation, ...]
    even_odd: bool


def number(value: float) -> str:
    if value == 0:
        value = 0.0
    rendered = f"{value:.10g}"
    if "e" not in rendered.lower() and "." not in rendered:
        rendered += ".0"
    return rendered


def tokenize(data: str) -> list[str]:
    tokens: list[str] = []
    end = 0
    for match in TOKEN_RE.finditer(data):
        if not SEPARATOR_RE.match(data[end : match.start()]):
            raise ConversionError(f"contenu de path non reconnu près de {data[end:match.start()]!r}")
        tokens.append(match.group())
        end = match.end()
    if not SEPARATOR_RE.match(data[end:]):
        raise ConversionError(f"contenu de path non reconnu près de {data[end:]!r}")
    return tokens


def parse_path(data: str) -> tuple[Operation, ...]:
    tokens = tokenize(data)
    operations: list[Operation] = []
    index = 0
    command: str | None = None
    current = (0.0, 0.0)
    subpath_start = current
    last_command = ""
    cubic_control: tuple[float, float] | None = None
    quadratic_control: tuple[float, float] | None = None

    def absolute_pair(x: float, y: float, relative: bool) -> tuple[float, float]:
        if relative:
            return current[0] + x, current[1] + y
        return x, y

    while index < len(tokens):
        token = tokens[index]
        if token.isalpha():
            command = token
            index += 1
            if command.upper() == "Z":
                operations.append(Operation("close"))
                current = subpath_start
                cubic_control = None
                quadratic_control = None
                last_command = "Z"
                command = None
                continue
        elif command is None:
            raise ConversionError("un nombre apparaît sans commande SVG")

        assert command is not None
        upper = command.upper()
        count = PARAMETERS[upper]
        if index + count > len(tokens) or any(value.isalpha() for value in tokens[index : index + count]):
            raise ConversionError(f"la commande {command} n'a pas assez de paramètres")
        values = [float(value) for value in tokens[index : index + count]]
        index += count
        relative = command.islower()

        if upper == "M":
            end = absolute_pair(values[0], values[1], relative)
            operations.append(Operation("move_to", end))
            current = end
            subpath_start = end
            last_command = "M"
            command = "l" if relative else "L"
        elif upper == "L":
            current = absolute_pair(values[0], values[1], relative)
            operations.append(Operation("line_to", current))
            last_command = "L"
        elif upper == "H":
            x = current[0] + values[0] if relative else values[0]
            current = (x, current[1])
            operations.append(Operation("line_to", current))
            last_command = "H"
        elif upper == "V":
            y = current[1] + values[0] if relative else values[0]
            current = (current[0], y)
            operations.append(Operation("line_to", current))
            last_command = "V"
        elif upper == "C":
            control_a = absolute_pair(values[0], values[1], relative)
            control_b = absolute_pair(values[2], values[3], relative)
            end = absolute_pair(values[4], values[5], relative)
            operations.append(Operation("cubic", (*end, *control_a, *control_b)))
            current = end
            cubic_control = control_b
            last_command = "C"
        elif upper == "S":
            if last_command in {"C", "S"} and cubic_control is not None:
                control_a = (2 * current[0] - cubic_control[0], 2 * current[1] - cubic_control[1])
            else:
                control_a = current
            control_b = absolute_pair(values[0], values[1], relative)
            end = absolute_pair(values[2], values[3], relative)
            operations.append(Operation("cubic", (*end, *control_a, *control_b)))
            current = end
            cubic_control = control_b
            last_command = "S"
        elif upper == "Q":
            control = absolute_pair(values[0], values[1], relative)
            end = absolute_pair(values[2], values[3], relative)
            operations.append(Operation("quadratic", (*end, *control)))
            current = end
            quadratic_control = control
            last_command = "Q"
        elif upper == "T":
            if last_command in {"Q", "T"} and quadratic_control is not None:
                control = (2 * current[0] - quadratic_control[0], 2 * current[1] - quadratic_control[1])
            else:
                control = current
            end = absolute_pair(values[0], values[1], relative)
            operations.append(Operation("quadratic", (*end, *control)))
            current = end
            quadratic_control = control
            last_command = "T"
        elif upper == "A":
            if values[3] not in {0.0, 1.0} or values[4] not in {0.0, 1.0}:
                raise ConversionError("les indicateurs large-arc et sweep doivent valoir 0 ou 1")
            end = absolute_pair(values[5], values[6], relative)
            operations.append(
                Operation("arc", (abs(values[0]), abs(values[1]), values[2], bool(values[3]), bool(values[4]), *end))
            )
            current = end
            last_command = "A"

        if upper not in {"C", "S"}:
            cubic_control = None
        if upper not in {"Q", "T"}:
            quadratic_control = None

    return tuple(operations)


def style_value(element: ET.Element, property_name: str) -> str | None:
    if property_name in element.attrib:
        return element.attrib[property_name].strip()
    for declaration in element.attrib.get("style", "").split(";"):
        name, separator, value = declaration.partition(":")
        if separator and name.strip() == property_name:
            return value.strip()
    return None


def read_svg(source: Path) -> tuple[tuple[float, float, float, float], tuple[SvgPath, ...]]:
    try:
        root = ET.parse(source).getroot()
    except (ET.ParseError, OSError) as error:
        raise ConversionError(f"impossible de lire {source}: {error}") from error

    raw_view_box = root.attrib.get("viewBox")
    if raw_view_box is None:
        raise ConversionError("le SVG doit définir un viewBox")
    values = re.findall(NUMBER, raw_view_box)
    if len(values) != 4:
        raise ConversionError("le viewBox doit contenir quatre nombres")
    view_box = tuple(float(value) for value in values)
    if view_box[2] <= 0 or view_box[3] <= 0:
        raise ConversionError("la largeur et la hauteur du viewBox doivent être positives")

    paths: list[SvgPath] = []

    def visit(element: ET.Element, transformed: bool = False) -> None:
        transformed = transformed or bool(element.attrib.get("transform", "").strip())
        tag = element.tag.rsplit("}", 1)[-1]
        if tag == "path":
            if transformed:
                raise ConversionError("les attributs transform ne sont pas encore pris en charge")
            fill = style_value(element, "fill")
            stroke = style_value(element, "stroke")
            if fill == "none" or (stroke is not None and stroke != "none"):
                raise ConversionError("seuls les paths remplis sont pris en charge (pas les strokes)")
            data = element.attrib.get("d", "").strip()
            if not data:
                raise ConversionError("un élément path ne possède pas d'attribut d")
            fill_rule = style_value(element, "fill-rule") or "nonzero"
            if fill_rule not in {"nonzero", "evenodd"}:
                raise ConversionError(f"fill-rule non pris en charge: {fill_rule}")
            paths.append(SvgPath(parse_path(data), fill_rule == "evenodd"))
        for child in element:
            visit(child, transformed)

    visit(root)
    if not paths:
        raise ConversionError("aucun élément path n'a été trouvé")
    return view_box, tuple(paths)


def inferred_component_name(source: Path) -> str:
    words = re.findall(r"[A-Za-z0-9]+", source.stem)
    name = "".join(word[:1].upper() + word[1:] for word in words)
    if not name:
        raise ConversionError("impossible de déduire un nom de composant depuis le fichier")
    return name if name.endswith("Icon") else f"{name}Icon"


def validate_component_name(name: str) -> None:
    if not re.fullmatch(r"[A-Z][A-Za-z0-9]*", name):
        raise ConversionError("le nom du composant doit être un identifiant Rust en PascalCase")


def point(values: tuple[float | bool, ...], offset: int = 0) -> str:
    return f"view.point({number(float(values[offset]))}, {number(float(values[offset + 1]))})"


def operation_lines(operation: Operation) -> list[str]:
    values = operation.values
    if operation.name in {"move_to", "line_to"}:
        return [f"path.{operation.name}({point(values)});"]
    if operation.name == "close":
        return ["path.close();"]
    if operation.name == "quadratic":
        return [f"path.curve_to({point(values)}, {point(values, 2)});"]
    if operation.name == "cubic":
        return [
            "path.cubic_bezier_to(",
            f"    {point(values)},",
            f"    {point(values, 2)},",
            f"    {point(values, 4)},",
            ");",
        ]
    if operation.name == "arc":
        return [
            "path.arc_to(",
            f"    view.radii({number(float(values[0]))}, {number(float(values[1]))}),",
            f"    px({number(float(values[2]))}),",
            f"    {str(values[3]).lower()},",
            f"    {str(values[4]).lower()},",
            f"    {point(values, 5)},",
            ");",
        ]
    raise AssertionError(f"unknown operation {operation.name}")


def generate(source: Path, component_name: str) -> str:
    view_box, paths = read_svg(source)
    uses_even_odd = any(path.even_odd for path in paths)
    uses_arc = any(operation.name == "arc" for path in paths for operation in path.operations)
    imports = "IntoElement, PathBuilder, Pixels, RenderOnce, Rgba, canvas, prelude::*"
    if uses_arc:
        imports = f"{imports}, px"
    if uses_even_odd:
        imports = f"FillOptions, FillRule, PathStyle, {imports}"

    if len(paths) == 1:
        fields = ["    color: Rgba,"]
        arguments = "width: Pixels, height: Pixels, color: Rgba"
        initialization = "            width,\n            height,\n            color,"
        additional_constructor: list[str] = []
        colors = ["self.color"]
    elif len(paths) == 2:
        fields = ["    first_color: Rgba,", "    second_color: Rgba,"]
        arguments = "width: Pixels, height: Pixels, color: Rgba"
        initialization = "            width,\n            height,\n            first_color: color,\n            second_color: color,"
        additional_constructor = [
            "",
            "    #[allow(dead_code)]",
            "    pub(crate) fn with_second_color(mut self, color: Rgba) -> Self {",
            "        self.second_color = color;",
            "        self",
            "    }",
        ]
        colors = ["self.first_color", "self.second_color"]
    else:
        fields = [f"    colors: [Rgba; {len(paths)}],"]
        arguments = "width: Pixels, height: Pixels, color: Rgba"
        initialization = f"            width,\n            height,\n            colors: [color; {len(paths)}],"
        additional_constructor = [
            "",
            "    #[allow(dead_code)]",
            f"    pub(crate) fn with_colors(width: Pixels, height: Pixels, colors: [Rgba; {len(paths)}]) -> Self {{",
            "        Self {",
            "            width,",
            "            height,",
            "            colors,",
            "        }",
            "    }",
        ]
        colors = [f"self.colors[{index}]" for index in range(len(paths))]

    lines = [
        f"use gpui::{{{imports}}};",
        "",
        "use super::ViewBox;",
        "",
        "#[derive(IntoElement)]",
        f"pub(crate) struct {component_name} {{",
        "    width: Pixels,",
        "    height: Pixels,",
        *fields,
        "}",
        "",
        f"impl {component_name} {{",
        f"    pub(crate) fn new({arguments}) -> Self {{",
        "        Self {",
        initialization,
        "        }",
        "    }",
        *additional_constructor,
        "}",
        "",
        f"impl RenderOnce for {component_name} {{",
        "    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {",
        "        canvas(",
        "            |_, _, _| {},",
        "            move |bounds, _, window, _| {",
        "                let view = ViewBox::new(",
        "                    bounds,",
        f"                    {number(view_box[0])},",
        f"                    {number(view_box[1])},",
        f"                    {number(view_box[2])},",
        f"                    {number(view_box[3])},",
        "                );",
    ]

    for index, svg_path in enumerate(paths):
        lines.append("")
        if svg_path.even_odd:
            lines.extend(
                [
                    "                let mut path = PathBuilder::fill().with_style(PathStyle::Fill(",
                    "                    FillOptions::default().with_fill_rule(FillRule::EvenOdd),",
                    "                ));",
                ]
            )
        else:
            lines.append("                let mut path = PathBuilder::fill();")
        for operation in svg_path.operations:
            for generated_line in operation_lines(operation):
                lines.append(f"                {generated_line}")
        lines.extend(
            [
                "",
                "                if let Ok(path) = path.build() {",
                f"                    window.paint_path(path, {colors[index]});",
                "                }",
            ]
        )

    lines.extend(
        [
            "            },",
            "        )",
            "        .w(self.width)",
            "        .h(self.height)",
            "    }",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def format_rust(source: str) -> str:
    try:
        result = subprocess.run(
            ["rustfmt", "--edition", "2024"],
            input=source,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        raise ConversionError(f"impossible de lancer rustfmt: {error}") from error
    if result.returncode != 0:
        message = result.stderr.strip() or "erreur inconnue"
        raise ConversionError(f"rustfmt a refusé le composant généré: {message}")
    return result.stdout


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("svg", type=Path, help="fichier SVG ou dossier source (sans sous-dossiers)")
    parser.add_argument("--name", help="nom Rust du composant (par défaut: nom du fichier + Icon)")
    output = parser.add_mutually_exclusive_group()
    output.add_argument("-o", "--output", type=Path, help="fichier Rust cible (stdout par défaut)")
    output.add_argument("--output-folder", type=Path, help="dossier cible pour les fichiers Rust")
    arguments = parser.parse_args()
    if arguments.svg.is_dir():
        if arguments.output_folder is None:
            parser.error("un dossier source nécessite --output-folder")
        if arguments.name is not None:
            parser.error("--name est réservé à la conversion d'un seul fichier")
    return arguments


def output_filename(source: Path) -> str:
    stem = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", source.stem)
    stem = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", stem)
    return "_".join(re.findall(r"[A-Za-z0-9]+", stem)).lower() + ".rs"


def convert_folder(source: Path, output_folder: Path) -> int:
    sources = sorted(path for path in source.iterdir() if path.is_file() and path.suffix.lower() == ".svg")
    if not sources:
        raise ConversionError(f"aucun fichier SVG trouvé dans {source}")

    destinations: dict[str, Path] = {}
    for svg in sources:
        filename = output_filename(svg)
        if filename in destinations:
            raise ConversionError(f"{destinations[filename]} et {svg} produisent le même fichier {filename}")
        destinations[filename] = svg

    failed = False
    for filename, svg in destinations.items():
        try:
            component_name = inferred_component_name(svg)
            validate_component_name(component_name)
            generated = format_rust(generate(svg, component_name))
            output_folder.mkdir(parents=True, exist_ok=True)
            destination = output_folder / filename
            destination.write_text(generated, encoding="utf-8")
            print(f"{svg} -> {destination}", file=sys.stderr)
        except (ConversionError, OSError) as error:
            print(f"erreur: {svg}: {error}", file=sys.stderr)
            failed = True
    return 1 if failed else 0


def main() -> int:
    arguments = parse_arguments()
    try:
        if arguments.svg.is_dir():
            return convert_folder(arguments.svg, arguments.output_folder)
        component_name = arguments.name or inferred_component_name(arguments.svg)
        validate_component_name(component_name)
        generated = format_rust(generate(arguments.svg, component_name))
        output = arguments.output
        if arguments.output_folder is not None:
            output = arguments.output_folder / output_filename(arguments.svg)
        if output is None:
            sys.stdout.write(generated)
        else:
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(generated, encoding="utf-8")
    except (ConversionError, OSError) as error:
        print(f"erreur: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
