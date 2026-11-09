# calcard-next

[![crates.io](https://img.shields.io/crates/v/calcard-next)](https://crates.io/crates/calcard-next)
[![build](https://github.com/EvanCarroll/calcard-next/actions/workflows/rust.yml/badge.svg)](https://github.com/EvanCarroll/calcard-next/actions/workflows/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/calcard-next)](https://docs.rs/calcard-next)
[![crates.io](https://img.shields.io/crates/l/calcard-next)](http://www.apache.org/licenses/LICENSE-2.0)

> [!IMPORTANT]
> **`calcard-next` is the official fork of [`calcard`](https://github.com/stalwartlabs/calcard).**
> Stalwart Labs archived the standalone `calcard` repository after `calcard` 0.3.14, when development moved into the [Stalwart repository](https://github.com/stalwartlabs/stalwart). From here on, `calcard-next` is maintained by **Evan Carroll** as an independent, standalone crate. Please file issues and pull requests at [EvanCarroll/calcard-next](https://github.com/EvanCarroll/calcard-next).
>
> To migrate, replace `calcard` with `calcard-next` in your `Cargo.toml` and `use calcard::` with `use calcard_next::` in your code.

`calcard-next` is a Rust crate for parsing, generating, and converting calendaring and contact data across multiple formats. It supports iCalendar and vCard, making it easy to work with `.ics` and `.vcf` files. It also fully handles JSCalendar and JSContact, the emerging standards commonly used in JMAP-based systems. In addition to parsing and serializing these formats, `calcard-next` provides seamless conversion between iCalendar and JSCalendar, as well as between vCard and JSContact, enabling smooth interoperability between traditional and modern calendaring and contact protocols.

In general, this library abides by the Postel's law or [Robustness Principle](https://en.wikipedia.org/wiki/Robustness_principle) which states that an implementation must be conservative in its sending behavior and liberal in its receiving behavior. This means that `calcard-next` will make a best effort to parse non-conformant iCal/vCard/JSCalendar/JSContact objects as long as these do not deviate too much from the standard.

## Features

Comprehensive features for working with calendaring and contact data, including:

- **iCalendar** support: Parsing and generating iCalendar data.
- **vCard** support: Parsing and generating vCard data.
- **JSCalendar** support: Parsing and generating JSCalendar data.
- **JSContact** support: Parsing and generating JSContact data.
- Seamless conversion between **iCalendar** and **JSCalendar**.
- Seamless conversion between **vCard** and **JSContact**.
- **Recurrence rules expansion**: Accurately computes and enumerates repeating events based on iCalendar and JSCalendar RRULEs.
- **IANA timezone detection**: Automatically resolves and handles custom and proprietary timezones.
- **Legacy character sets**: Decodes vCard 2.1 and vCalendar 1.0 text declared in non-UTF-8 character sets (`CHARSET=Shift_JIS`, `ISO-8859-1`, ...). This is the default `legacy-charsets` feature; size-sensitive builds such as WebAssembly can disable it, in which case such text is preserved as binary data.
- **Modern date-time types**: Built on [`jiff`](https://docs.rs/jiff), with a bundled IANA time zone database for reproducible results on every platform.

## Usage

### Parsing an iCalendar and/or vCard stream

You can parse vCard and iCalendar files using the stream-based parser:

```rust
let input = "BEGIN:VCARD\nVERSION:3.0\nFN:John Doe\nEND:VCARD\n";
let mut parser = Parser::new(&input);

loop {
    match parser.entry() {
        Entry::VCard(vcard) => println!("Parsed VCard: {:?}", vcard),
        Entry::ICalendar(ical) => println!("Parsed ICalendar: {:?}", ical),
        Entry::InvalidLine(line) => eprintln!("Invalid line found: {}", line),
        Entry::UnexpectedComponentEnd { expected, found } => {
            eprintln!("Unexpected end: expected {:?}, found {:?}", expected, found);
        },
        Entry::UnterminatedComponent(component) => {
            eprintln!("Unterminated component: {:?}", component);
        },
        Entry::Eof => {
            break;
        }
    }
}
```

### Parsing iCalendar

You can parse a single iCalendar using the `ICalendar::parse` method:

```rust
let input = "BEGIN:VCALENDAR\nVERSION:2.0\nEND:VCALENDAR\n";
let ical = ICalendar::parse(&input);

println!("Parsed ICalendar: {:?}", ical);
```

### Parsing vCard

You can parse a single vCard using the `VCard::parse` method:

```rust
let input = "BEGIN:VCARD\nVERSION:3.0\nFN:John Doe\nEND:VCARD\n";
let vcard = VCard::parse(&input);

println!("Parsed VCard: {:?}", vcard);
```
### Parsing JSCalendar

You can parse a JSCalendar JSON string using the `JSCalendar::parse` method:

```rust
let input = r#"{
                    "@type": "Group",
                    "uid": "bf0ac22b-4989-4caf-9ebd-54301b4ee51a",
                    "updated": "2020-01-15T18:00:00Z",
                    "title": "A simple group",
                    "entries": [{
                        "@type": "Event",
                        "uid": "a8df6573-0474-496d-8496-033ad45d7fea",
                        "updated": "2020-01-02T18:23:04Z",
                        "title": "Some event",
                        "start": "2020-01-15T13:00:00",
                        "timeZone": "America/New_York",
                        "duration": "PT1H"
                    },
                    {
                        "@type": "Task",
                        "uid": "2a358cee-6489-4f14-a57f-c104db4dc2f2",
                        "updated": "2020-01-09T14:32:01Z",
                        "title": "Do something"
                    }]
                }"#;

let jscalendar = JSCalendar::<String, String>::parse(input).unwrap();
println!("Parsed JSCalendar: {}", jscalendar.to_string_pretty());
```

### Parsing JSContact

You can parse a JSContact JSON string using the `JSContact::parse` method:

```rust
let input = r#"{
                    "@type": "Card",
                    "version": "1.0",
                    "uid": "22B2C7DF-9120-4969-8460-05956FE6B065",
                    "kind": "individual",
                    "name": {
                    "components": [
                        { "kind": "given", "value": "John" },
                        { "kind": "surname", "value": "Doe" }
                    ],
                    "isOrdered": true
                    }
                }"#;

let jscontact = JSContact::<String, String>::parse(input).unwrap();
println!("Parsed JSContact: {}", jscontact.to_string_pretty());
```

### Converting to/from iCalendar and JSCalendar

To convert a JSCalendar to an iCalendar, use the `JSCalendar::into_icalendar` method:

```rust
let ical = JSCalendar::<String, String>::parse(input).unwrap().into_icalendar().unwrap();
```

To convert an iCalendar to a JSCalendar, use the `ICalendar::into_jscalendar` method:

```rust
let jscalendar = ICalendar::parse(&input).unwrap().into_jscalendar::<String, String>().unwrap();
```

### Converting to/from vCard and JSContact

To convert a JSContact to a vCard, use the `JSContact::into_vcard` method:

```rust
let vcard = JSContact::<String, String>::parse(input).unwrap().into_vcard().unwrap();
```

To convert a vCard to a JSContact, use the `VCard::into_jscontact` method:

```rust
let jscontact = VCard::parse(&input).unwrap().into_jscontact::<String, String>().unwrap();
```

### Generating iCalendar and vCard

To generate a vCard or iCalendar, simply call `.to_string()` on the parsed object:

```rust
let vcard_string = vcard.to_string();
println!("Generated vCard:\n{}", vcard_string);

let ical_string = ical.to_string();
println!("Generated ICalendar:\n{}", ical_string);
```

*Note: Documentation for creating VCard and ICalendar objects is coming soon.*

### Generating JSCalendar and JSContact

To generate a JSCalendar or JSContact, simply call `.to_string()` on the parsed object or `.to_string_pretty()` for a more human-readable format:

```rust
let jscalendar_string = jscalendar.to_string();
println!("Generated JSCalendar:\n{}", jscalendar_string);

let jscontact_string = jscontact.to_string();
println!("Generated JSContact:\n{}", jscontact_string);
```

*Note: Documentation for creating JSCalendar and JSContact objects is coming soon.*


## Testing and Fuzzing

To run the testsuite:

```bash
 $ cargo test --all-features
```

or, to run the testsuite with MIRI:

```bash
 $ cargo +nightly miri test --all-features
```

To fuzz the library with `cargo-fuzz`:

```bash
 $ cargo +nightly fuzz run fuzz_all
 $ cargo +nightly fuzz run fuzz_random
 $ cargo +nightly fuzz run fuzz_structured
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Maintainer

`calcard-next` is maintained by Evan Carroll.

## History and Funding

`calcard-next` is a fork of `calcard`, originally written by [Stalwart Labs](https://stalw.art).

## Copyright

Copyright (C) 2026, Evan Carroll (`calcard-next` modifications)

Copyright (C) 2020, Stalwart Labs LLC (original `calcard` code)
