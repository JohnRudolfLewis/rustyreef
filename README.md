# Rusty Reef

A RaspberryPi based reef controller implemented in Rust where the outputs are fully user programmable via a tiny subset of Lisp called Risp (Reef-Lisp).

## Why

- **Why do you need a reef controller?** My saltwater reef aquarium has many sensors/pumps/lights. It needs automation, and I want all automation in one interface.
- **Why not use a commercial controller like the Neptune Apex?** I am a gearhead, I like gadgets. But I like building my own gadgets even better.
- **Why not use or extend [reef-pi](https://reef-pi.github.io/)?** I have one currently. I don't like its UI, nor its lack of programability. Also, I want to use devices such as the Whitebox Labs Tentacle T3 and EZO devices for measurements, and Z-Wave Power Strips for control/query of pumps, etc. These devices are not supported. It would require nearly a complete re-write to implement what I want. And what I want is not inline with the tennants that the original author has set out.
- **Why Rust?** For the learning experience. I did not know the language when I started this project.
- **Why Lisp?** Because. Just because.

## Project Status

Nacent. Just getting started. Do not use any of this yet.

![Rust](https://github.com/JohnRudolfLewis/rustyreef/workflows/Rust/badge.svg?branch=master)

### TODO

Major systems
- [ ] Minimal Lisp Implementation.
- [ ] Input and Output abstractions.
- [ ] Main program loops.
- [ ] Config file integration.
- [ ] Minimal REST Service and UI

I/O Integration
- [ ] Device driver for Whitebox Labs EZO-RTD sensor to monitor temperature.
- [ ] Device driver for Whitebox Labs EZO-pH sensor to monitor pH.
- [ ] Device driver for Whitebox Labs EZO-EC sensor to monitor salinity.
- [ ] Device driver for Adafruit DC/Stepper Motor HAT to drive peristaltic dosing pumps.
- [ ] Device driver for MCP4728 Quad DAC based 0-10v to control LED lighting fixture.
- [ ] Integrate USB Z-Wave dongle for control and query of Z-Wave power strips.


## How it works

- ReefState, a HashMap, contains all the values read from inputs, and all the expected output values to be written to outputs.

- Inputs are config file driven. ReefState gets updated from the input at a rate appropriate to the specific input.

- Outputs are config file driven. The output is updated from ReefState at a rate appropriate to the specific output.

- Each output has an associated Lisp-like expression called a Risp that gets used to determine the output value.

- Each output also updates a ReefState value each time it changes.

- Some Outputs are virtual, there is no device to be updated. They are used elsewhere by other Risp expressions.

- Upon every tick of a calculation loop, for each configured output, ReefState is updated from the evaluation of the Risp expression.

### Example Risp Expressions:

#### Light_Outlet ####
```
(if (> 06:00:00 (now) 18:00:00) 1 0)
```
The outlet will turn on only when the current time is between 6:00am and 6:00pm local.

#### Heater_Outlet ####
```
(cond 
    ((Tank_Temperature > 82) (0))
    ((Tank_Temperature < 78) (1))
    (t Heater_Outlet)
)
```
The outlet will turn off when the temperature exceeds 82 and turn off when the temperature is below 78.

#### Feed_Mode_Virtual ####
```
(cond 
    ((and (== Feed_Mode_Button 1) (== Feed_Mode_Virtual false) true ) true)
    ((and (== Feed_Mode_Virtual true) ( > (- (now) Feed_Mode_Virtual_Changed) PT5M) true ) false)
    (t Feed_Mode_Virtual)
)
```

If the button is pressed and the virtual is false, set the virtual to true. Which also sets a variable for when it changed.
If the the virtual is true and more than 5 minutes have passed since it changed, set the virtual to false.

#### Return_Pump_Outlet ####
```
(if Feed_Mode_Virtual 0 1)
```
The outlet will be off when feed mode is on.

#### Wave_Pump_Outlet ####
```
(if Feed_Mode_Virtual 0 1)
```
Same as return pump.
