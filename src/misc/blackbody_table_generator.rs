/*
    This file is part of Infinite Escape Velocity.

    Infinite Escape Velocity is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Infinite Escape Velocity is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Infinite Escape Velocity.  If not, see <https://www.gnu.org/licenses/>.
*/

type ColorTriple = (f64, f64, f64);

fn main() {
    for temp in (1000..12000).step_by(440) {
        let color = color_at_temperature(temp as f64);

        let r = (color.0 * 255.0) as i32;
        let g = (color.1 * 255.0) as i32;
        let b = (color.2 * 255.0) as i32;

        println!("vec3({r}, {g}, {b}),");
    }
}

fn color_at_temperature(temperature: f64) -> ColorTriple {
    let CIE = integrate_CIE_tristimulus(temperature);
    let sRGB = CIE_to_sRGB(CIE);

    let normalize = 1.0 / sRGB.0.max(sRGB.1.max(sRGB.2));
    return (sRGB.0 * normalize, sRGB.1 * normalize, sRGB.2 * normalize);
}

fn integrate_CIE_tristimulus(temperature: f64) -> ColorTriple {
    let mut X: f64 = 0.0;
    let mut Y: f64 = 0.0;
    let mut Z: f64 = 0.0;

    for wavelength in 380..780 {
        let meter_wavelength = wavelength as f64 / 1e9;
        let irradiance =
            blackbody_radiance_at_frequency(meter_wavelength as f64, temperature as f64);
        let stimulus = CIE_stimulus(wavelength as f64);

        X += stimulus.0 * irradiance;
        Y += stimulus.1 * irradiance;
        Z += stimulus.2 * irradiance;
    }

    let normalize = 1.0 / Y;

    return (X * normalize, Y * normalize, Z * normalize);
}

fn blackbody_radiance_at_frequency(wavelength: f64, temperature: f64) -> f64 {
    const PLANCK_CONSTANT: f64 = 6.62607015e-34;
    const BOLTZMANN_CONSTANT: f64 = 1.380649e-23;
    const C: f64 = 299792458.0;

    let left = (C.powi(2) * PLANCK_CONSTANT * 2.0) / wavelength.powi(5);
    let exponent = (PLANCK_CONSTANT * C) / (wavelength * BOLTZMANN_CONSTANT * temperature);
    let right = 1.0 / (exponent.exp() - 1.0);

    return (left * right);
}

fn CIE_to_sRGB(colors: ColorTriple) -> ColorTriple {
    let x = colors.0;
    let y = colors.1;
    let z = colors.2;

    let red_linear = (3.2406 * x) + (-1.5372 * x) + (-0.4986 * x);
    let green_linear = (-0.9689 * y) + (1.8758 * y) + (0.0415 * y);
    let blue_linear = (0.0557 * z) + (-0.2042 * z) + (1.0570 * z);

    return (
        delinearize_sRGB(red_linear),
        delinearize_sRGB(green_linear),
        delinearize_sRGB(blue_linear),
    );
}

fn delinearize_sRGB(color: f64) -> f64 {
    if color <= 0.0031308 {
        return color * 12.92;
    } else {
        return (1.055 * color.powf(1.0 / 2.4)) - 0.055;
    }
}

fn CIE_stimulus(wavelength: f64) -> ColorTriple {
    let x_bar = (1.056 * CIE_gaussian(wavelength, 599.8, 37.9, 31.0))
        + (0.362 * CIE_gaussian(wavelength, 442.0, 16.0, 26.7))
        + (-0.065 * CIE_gaussian(wavelength, 501.1, 20.4, 26.2));

    let y_bar = (0.821 * CIE_gaussian(wavelength, 568.8, 46.9, 40.5))
        + (0.286 * CIE_gaussian(wavelength, 530.9, 16.3, 31.1));

    let z_bar = (1.217 * CIE_gaussian(wavelength, 437.0, 11.8, 36.0))
        + (0.681 * CIE_gaussian(wavelength, 459.0, 26.0, 13.8));

    return (x_bar, y_bar, z_bar);
}

fn CIE_gaussian(wavelength: f64, peak: f64, left: f64, right: f64) -> f64 {
    let identical = -0.5 * (wavelength - peak).powi(2);
    if wavelength < peak {
        return (identical / left.powi(2)).exp();
    } else {
        return (identical / right.powi(2)).exp();
    }
}
