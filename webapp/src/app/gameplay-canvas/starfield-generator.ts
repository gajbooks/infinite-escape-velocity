import Konva from "konva";

const SPECTRA_TABLE: string[] = [
    "#FFB10E",
    "#FFCF54",
    "#FFDE8F",
    "#FFE4B8",
    "#FFE7D4",
    "#FFE8E8",
    "#FFE8F6",
    "#FCE6FF",
    "#F4DFFF",
    "#EFD9FF",
    "#EAD5FF",
    "#E7D2FE",
    "#E4CFFF",
    "#E2CDFF",
    "#E0CBFE",
    "#DEC9FF",
    "#DDC8FF",
    "#DCC6FF",
    "#DBC5FF",
    "#DAC4FF",
    "#D9C4FF",
    "#D8C3FF",
    "#D8C2FF",
    "#D7C2FF",
];

function simple_xorshift(state: number): number {
    let x = state;
    x ^= x << 13;
    x ^= x >>> 7;
    x ^= x << 17;
    return x;
}

function zero_one_generator(random: number): number {
    // Ensure floating point division is done by adding a fraction
    return (Math.abs(random) + .0001) / Math.pow(2, 31);
}

function hash_coordinates(x: number, y: number): number {
    let seed = (x << 16) | ((y << 16) >>> 16);
    for (let i = 0; i < 5; i++) {
        seed = simple_xorshift(seed);
    }

    return seed;
}

function star_from_coordinates(x: number, y: number, coordinates_per_cell: number): Star {
    let seed = hash_coordinates(x, y);

    let x_offset = zero_one_generator(seed);

    seed = simple_xorshift(seed);
    let y_offset = zero_one_generator(seed);

    seed = simple_xorshift(seed);
    let speed_multiplier = zero_one_generator(seed);

    seed = simple_xorshift(seed);
    let spectra = zero_one_generator(seed);

    seed = simple_xorshift(seed);
    let size = zero_one_generator(seed);

    let screen_x = (x - 1.0 + x_offset) * coordinates_per_cell;
    let screen_y = (y - 1.0 + y_offset) * coordinates_per_cell;

    return {
        screen_x: screen_x,
        screen_y: screen_y,
        speed_multiplier: speed_multiplier,
        spectra: spectra,
        size: speed_multiplier
    };
}

interface Star {
    screen_x: number;
    screen_y: number;
    speed_multiplier: number;
    size: number;
    spectra: number;
}

export class StarfieldGenerator {
    star_list: Star[] = [];
    star_graphics: Konva.Circle[] = [];
    star_layer: Konva.Layer;
    min_coordinate_divisor: number = 2.0;
    max_coordinate_divisor: number = 15.0;
    world_coordinates_per_cell: number = 60;
    max_size: number = 1.5;
    min_size: number = .5;


    constructor(star_layer: Konva.Layer) {
        this.star_layer = star_layer;
    }

    draw_stars(
        x_coordinate: number,
        y_coordinate: number,
        screen_width: number,
        screen_height: number,
    ) {
        x_coordinate = x_coordinate / this.max_coordinate_divisor;
        y_coordinate = y_coordinate / this.max_coordinate_divisor;

        let start_x = Math.floor(x_coordinate / this.world_coordinates_per_cell);
        let start_y = Math.floor(y_coordinate / this.world_coordinates_per_cell);

        let cells_x = (screen_width / this.world_coordinates_per_cell) + 1;
        let cells_y = (screen_height / this.world_coordinates_per_cell) + 1;

        this.generate_stars(
            Math.round(start_x),
            Math.round(start_y),
            Math.round(cells_x),
            Math.round(cells_y),
        );

        let divisor_ratio = (this.max_coordinate_divisor / this.min_coordinate_divisor) - 1.0;

        let list_length_difference = this.star_list.length - this.star_graphics.length;
        if (list_length_difference > 0) {
            for (let i = 0; i < list_length_difference; i++) {
                let star = new Konva.Circle({
                    strokeEnabled: false,
                    visible: false,
                    perfectDrawEnabled: false,
                    shadowForStrokeEnabled: false,
                    listening: false,
                });
                this.star_layer.add(star);
                this.star_graphics.push(star);
            }
        } else if (list_length_difference < 0) {
            for (let i = 0; i < Math.abs(list_length_difference); i++) {
                this.star_graphics.pop()?.destroy();
            }
        }

        for (let i = 0; i < this.star_list.length; i++) {
            let star = this.star_list[i];
            let graphics = this.star_graphics[i];
            let offset_ratio = 1.0 + (divisor_ratio * star.speed_multiplier);
            let star_x = (star.screen_x - x_coordinate) * offset_ratio;
            let star_y = (star.screen_y - y_coordinate) * offset_ratio;

                let size = ((this.max_size - this.min_size) * star.size) + this.min_size;
                graphics.x(star_x);
                graphics.y(star_y);
                graphics.radius(size);
                graphics.visible(true);
                graphics.fill(SPECTRA_TABLE[Math.floor(star.spectra * SPECTRA_TABLE.length)]);
        }
    }

    generate_stars(start_x: number, start_y: number, cells_x: number, cells_y: number) {
        this.star_list = [];
        for (let i = 0; i < cells_y; i++) {
            for (let j = 0; j < cells_x; j++) {
                    this.star_list.push(star_from_coordinates(j + start_x, i + start_y, this.world_coordinates_per_cell));
            }
        }
    }
}
