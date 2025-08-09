// Assignment 1: Temperature Converter
fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    (c * 9.0 / 5.0) + 32.0
}

// Assignment 2: Number Analyzer
fn is_even(n: i32) -> bool {
    n % 2 == 0
}

// Assignment 3: Guessing Game
fn check_guess(guess: i32, secret: i32) -> i32 {
    if guess == secret {
        0
    } else if guess > secret {
        1
    } else {
        -1
    }
}

fn main() {
    // ------------------------
    // Assignment 1: Temp Converter
    println!("--- Assignment 1: Temperature Converter ---");
    let temp_f = 32;
    let temp_c = fahrenheit_to_celsius(temp_f as f64);
    println!("{temp_f}°F = {temp_c:.2}°C");

    for i in 1..=5 {
        let next_temp_f = temp_f + i;
        let next_temp_c = fahrenheit_to_celsius(next_temp_f as f64);
        println!("{next_temp_f}°F = {next_temp_c:.2}°C");
    }

    // Extra: convert 100°C to Fahrenheit using the unused function
    println!("100°C = {:.2}°F", celsius_to_fahrenheit(100.0));

    // ------------------------
    // Assignment 2: Number Analyzer
    println!("\n--- Assignment 2: Number Analyzer ---");
    let numbers = [3, 5, 7, 10, 15, 22, 9, 8, 13, 30];

    for &n in &numbers {
        if n % 3 == 0 && n % 5 == 0 {
            println!("{n}: FizzBuzz");
        } else if n % 3 == 0 {
            println!("{n}: Fizz");
        } else if n % 5 == 0 {
            println!("{n}: Buzz");
        } else if is_even(n) {
            println!("{n}: Even");
        } else {
            println!("{n}: Odd");
        }
    }

    let mut sum = 0;
    let mut i = 0;
    while i < numbers.len() {
        sum += numbers[i];
        i += 1;
    }
    println!("Sum of array: {sum}");

    let mut max = numbers[0];
    for &n in &numbers {
        if n > max {
            max = n;
        }
    }
    println!("Largest number: {max}");

    // ------------------------
    // Assignment 3: Guessing Game
    println!("\n--- Assignment 3: Guessing Game ---");
    let secret = 17;
    let guesses = [10, 20, 15, 17];
    let mut attempts = 0;

    for guess in guesses {
        attempts += 1;
        match check_guess(guess, secret) {
            0 => {
                println!("Guess {guess} is correct! Attempts: {attempts}");
                break;
            }
            1 => println!("Guess {guess} is too high."),
            -1 => println!("Guess {guess} is too low."),
            _ => (),
        }
    }
}
