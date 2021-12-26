fn main() {
    std::fs::read("data/Hello.class")
        .ok()
        .map(std::io::Cursor::new)
        .map(|data| {
            (
                data,
                (
                    // read_u8
                    (move |d| {
                        Some([0])
                            .and_then(|mut buf| std::io::Read::read_exact(d, &mut buf).map(|_| u8::from_be(buf[0])).ok())
                            .expect("read u8")
                    }) as fn(&mut std::io::Cursor<Vec<u8>>) -> u8,
                    // read_u16
                    (move |d| {
                        Some([0, 0])
                            .and_then(|mut buf| std::io::Read::read_exact(d, &mut buf).map(|_| u16::from_be_bytes(buf)).ok())
                            .expect("read u16")
                    }) as fn(&mut std::io::Cursor<Vec<u8>>) -> u16,
                    // read_u32
                    (move |d| {
                        Some([0, 0, 0, 0])
                            .and_then(|mut buf| std::io::Read::read_exact(d, &mut buf).map(|_| u32::from_be_bytes(buf)).ok())
                            .expect("read u32")
                    }) as fn(&mut std::io::Cursor<Vec<u8>>) -> u32,
                    // read_i32
                    (move |d| {
                        Some([0, 0, 0, 0])
                            .and_then(|mut buf| std::io::Read::read_exact(d, &mut buf).map(|_| i32::from_be_bytes(buf)).ok())
                            .expect("read i32")
                    }) as fn(&mut std::io::Cursor<Vec<u8>>) -> i32,
                    // read_f32
                    (move |d| {
                        Some([0, 0, 0, 0])
                            .and_then(|mut buf| std::io::Read::read_exact(d, &mut buf).map(|_| f32::from_be_bytes(buf)).ok())
                            .expect("read f32")
                    }) as fn(&mut std::io::Cursor<Vec<u8>>) -> f32,
                ),
            )
        })
        .map(|(mut data, (read_u8, read_u16, read_u32, read_i32, read_f32))| {
            (
                (read_u32(&mut data), read_u16(&mut data), read_u16(&mut data)),
                data,
                (read_u8, read_u16, read_u32, read_i32, read_f32),
            )
        })
        .map(|((magic, ..), data, read)| Some(|| assert!(magic == 0xCAFEBABE)).map(|f| f()).map(|_| (data, read)).unwrap())
        .map(|(mut data, (read_u8, read_u16, read_u32, read_i32, read_f32))| {
            Some(read_u16(&mut data) as usize - 1)
                .map(|n| {
                    std::iter::repeat_with(|| match read_u8(&mut data) {
                        1 => (
                            (
                                std::iter::repeat(0)
                                    .take(read_u16(&mut data) as usize)
                                    .map(Some)
                                    .collect::<Option<Vec<_>>>()
                                    .and_then(|mut buf| std::io::Read::read_exact(&mut data, &mut buf).map(|_| buf).ok())
                                    .map(String::from_utf8)
                                    .transpose()
                                    .ok()
                                    .flatten(),
                                None,
                                None,
                                None,
                                None,
                                None,
                            ),
                            <_>::default(),
                            <_>::default(),
                        ),
                        3 => ((None, None, Some(read_i32(&mut data)), None, None, None), <_>::default(), <_>::default()),
                        4 => ((None, None, None, Some(read_f32(&mut data)), None, None), <_>::default(), <_>::default()),
                        5 => (
                            (None, None, None, None, Some((read_i32(&mut data), read_i32(&mut data))), None),
                            <_>::default(),
                            <_>::default(),
                        ),
                        6 => (
                            (None, None, None, None, None, Some((read_f32(&mut data), read_f32(&mut data)))),
                            <_>::default(),
                            <_>::default(),
                        ),
                        7 => (<_>::default(), Some(read_u16(&mut data)), <_>::default()),
                        8 => ((None, Some(read_u16(&mut data)), None, None, None, None), <_>::default(), <_>::default()),
                        9 => (
                            <_>::default(),
                            <_>::default(),
                            (Some((read_u16(&mut data), read_u16(&mut data))), None, None, None, None, None, None),
                        ),
                        10 => (
                            <_>::default(),
                            <_>::default(),
                            (None, Some((read_u16(&mut data), read_u16(&mut data))), None, None, None, None, None),
                        ),
                        11 => (
                            <_>::default(),
                            <_>::default(),
                            (None, None, Some((read_u16(&mut data), read_u16(&mut data))), None, None, None, None),
                        ),
                        12 => (
                            <_>::default(),
                            <_>::default(),
                            (None, None, None, Some((read_u16(&mut data), read_u16(&mut data))), None, None, None),
                        ),
                        15 => (
                            <_>::default(),
                            <_>::default(),
                            (None, None, None, None, Some((read_u8(&mut data), read_u16(&mut data))), None, None),
                        ),
                        16 => (<_>::default(), <_>::default(), (None, None, None, None, None, Some(read_u16(&mut data)), None)),
                        18 => (
                            <_>::default(),
                            <_>::default(),
                            (None, None, None, None, None, None, Some((read_u16(&mut data), read_u16(&mut data)))),
                        ),
                        e => unimplemented!("unknown tag: {:02}", e),
                    })
                    .take(n)
                    .collect::<Vec<(
                        (
                            Option<String>,     // utf8
                            Option<u16>,        // string
                            Option<i32>,        // int
                            Option<f32>,        // float
                            Option<(i32, i32)>, // long
                            Option<(f32, f32)>, // double
                        ),
                        Option<u16>, // class
                        (
                            Option<(u16, u16)>, // field ref
                            Option<(u16, u16)>, // method ref
                            Option<(u16, u16)>, // interface method ref
                            Option<(u16, u16)>, // name and type
                            Option<(u8, u16)>,  // method handle
                            Option<u16>,        // method type
                            Option<(u16, u16)>, // invoke dynamic
                        ),
                    )>>()
                })
                .map(|constants| (data, (read_u8, read_u16, read_u32, read_i32, read_f32), constants))
                .unwrap()
        })
        .and_then(|(mut data, (read_u8, read_u16, read_u32, read_i32, read_f32), constants)| {
            std::iter::repeat_with(|| read_u16(&mut data))
                .take(5) // ignore attributes (10 bytes worth)
                .last()
                .map(|_| (data, (read_u8, read_u16, read_u32, read_i32, read_f32), constants))
        })
        .map(|(mut data, (read_u8, read_u16, read_u32, read_i32, read_f32), constants)| {
            Some(read_u16(&mut data))
                .map(|n| (n, &mut data, &constants))
                .map(|(n, data, constants)| {
                    std::iter::repeat_with(move || {
                        Some((read_u16(data), read_u16(data), read_u16(data), read_u16(data)))
                            .and_then(|(_access_flags, name_index, descriptor_index, attributes_count)| {
                                constants
                                    .get(name_index as usize - 1)
                                    .map(|((name, ..), ..)| {
                                        // TODO assert static entry point
                                        name.clone().unwrap()
                                    })
                                    .and_then(|name| {
                                        constants
                                            .get(descriptor_index as usize - 1)
                                            .map(|((name, ..), ..)| name.clone().unwrap())
                                            .map(|descriptor| (name, descriptor))
                                    })
                                    .map(|(name, descriptor)| {
                                        std::iter::repeat_with(|| {
                                            Some((
                                                read_u16(data),
                                                std::iter::repeat(0)
                                                    .take(read_u32(data) as usize)
                                                    .map(Some)
                                                    .collect::<Option<Vec<_>>>()
                                                    .and_then(|mut buf| std::io::Read::read_exact(data, &mut buf).map(|_| buf).ok())
                                                    .map(std::io::Cursor::new)
                                                    .unwrap(),
                                            ))
                                            .and_then(|(name_index, mut attributes)| {
                                                matches!(constants.get(name_index as usize - 1), Some(((Some(name), ..), ..)) if name == "Code").then(|| {
                                                    (
                                                        read_u16(&mut attributes),
                                                        read_u16(&mut attributes),
                                                        std::iter::repeat(0)
                                                            .take(read_u32(&mut attributes) as usize)
                                                            .map(Some)
                                                            .collect::<Option<Vec<_>>>()
                                                            .and_then(|mut buf| std::io::Read::read_exact(&mut attributes, &mut buf).map(|_| buf).ok())
                                                            .unwrap_or_default(),
                                                    )
                                                })
                                            })
                                        })
                                        .flatten()
                                        .take(attributes_count as usize)
                                        .last()
                                        .map(|(max_stack, max_locals, code)| (max_stack, max_locals, code, name, descriptor))
                                        .unwrap_or_default()
                                    })
                            })
                            .unwrap()
                    })
                    .take(n as usize)
                    .collect::<Vec<_>>()
                })
                .map(|methods| (data, (read_u8, read_u16, read_u32, read_i32, read_f32), constants, methods))
                .unwrap()
        })
        .and_then(|(_data, _read, constants, methods)| {
            methods
                .iter()
                .find(|(.., name, descriptor)| name == "main" && descriptor == "([Ljava/lang/String\x3b)V")
                .map(|(max_stack, max_locals, code, _name, _descriptor)| {
                    Some(std::iter::repeat(0).take(*max_locals as usize).collect::<Vec<_>>()).map(|_locals| {
                        Some(0)
                            .map(|pc| {
                                (
                                    pc,
                                    (
                                        (move |(s, sp), t| Some(s[*sp as usize] = t).map(|_| *sp += 1).map(drop).unwrap())
                                            as fn((&mut Vec<(Option<i32>, Option<String>)>, &mut u16), (Option<i32>, Option<String>)),
                                        (move |(s, sp)| Some(*sp = sp.saturating_sub(1)).map(|_| s[*sp as usize].clone()).unwrap())
                                            as fn((&mut Vec<(Option<i32>, Option<String>)>, &mut u16)) -> (Option<i32>, Option<String>),
                                    ),
                                    0,
                                    std::iter::repeat((None, None)).take(*max_stack as usize).collect::<Vec<(Option<i32>, Option<String>)>>(),
                                )
                            })
                            .map(|(mut pc, (push, pop), mut sp, mut stack)| {
                                std::iter::from_fn(|| {
                                    (pc < code.len()).then(|| {
                                        Some(code[pc]).and_then(|op| Some(pc += 1).map(|_| op)).and_then(|op| {
                                            Some(match op {
                                                0x00 => { /* NOP */ }
                                                0x02 | 0x03 | 0x04 => push((&mut stack, &mut sp), (Some((op - 3) as i32), None)),
                                                0x12 => {
                                                    /* LDC */
                                                    Some(code[pc])
                                                        .and_then(|b| Some(pc += 1).map(|_| b))
                                                        .and_then(|index| constants.get(index as usize - 1))
                                                        .map(|c| {
                                                            Some(match c {
                                                                ((Some(s), ..), ..) => (Some(s.clone()), None),
                                                                ((_, Some(s), ..), ..) => match constants.get(*s as usize - 1).unwrap() {
                                                                    ((Some(s), ..), ..) => (Some(s.clone()), None),
                                                                    _ => {
                                                                        unimplemented!("type is not supported")
                                                                    }
                                                                },
                                                                ((_, _, Some(i), ..), ..) => (None, Some(*i as i32)),
                                                                _ => unimplemented!("type is not supported"),
                                                            })
                                                            .map(|(s, i)| push((&mut stack, &mut sp), (i, s)))
                                                        })
                                                        .map(drop)
                                                        .unwrap()
                                                }
                                                0xb1 => return None,
                                                0xb2 => pc += 2,
                                                0xb6 => {
                                                    /* INVOKEVIRTUAl */
                                                    Some(pop((&mut stack, &mut sp)))
                                                        .map(|_res| {
                                                            Some((code[pc], code[pc + 1]))
                                                                .map(|(l, r)| u16::from_be_bytes([l, r]))
                                                                .and_then(|d| Some(pc += 2).map(|_| d))
                                                                .map(|d| {
                                                                    constants.get(d as usize - 1).map(|c| match c {
                                                                        (.., (_, Some((_, ty)), ..)) => constants
                                                                            .get(*ty as usize - 1)
                                                                            .map(|c| match c {
                                                                                (.., (_, _, _, Some((name, descriptor)), ..)) => (*name, *descriptor),
                                                                                e => unreachable!("{:#?}", e),
                                                                            })
                                                                            .map(|(name, descriptor)| {
                                                                                (
                                                                                    constants.get(name as usize - 1).unwrap().0 .0.clone().unwrap(),
                                                                                    constants.get(descriptor as usize - 1).unwrap().0 .0.clone().unwrap(),
                                                                                )
                                                                            })
                                                                            .and_then(|(name, descriptor)| {
                                                                                methods
                                                                                    .iter()
                                                                                    .find(|(.., n, d)| *n == name && *d == descriptor)
                                                                                    .map(|method| (Some(method), None))
                                                                                    .or_else(|| {
                                                                                        Some((
                                                                                            None,
                                                                                            match &*name {
                                                                                                "print" => Some(Box::new(move |args: &[(Option<i32>, Option<String>)]| {
                                                                                                    args.iter().for_each(|arg| match arg {
                                                                                                        (Some(s), ..) => print!("{}", s),
                                                                                                        (.., Some(i)) => print!("{}", i),
                                                                                                        _ => unreachable!(),
                                                                                                    })
                                                                                                })
                                                                                                    as Box<dyn for<'e> Fn(&'e [(Option<i32>, Option<String>)])>),
                                                                                                "println" => Some(Box::new(move |args: &[(Option<i32>, Option<String>)]| {
                                                                                                    args.iter().for_each(|arg| match arg {
                                                                                                        (Some(i), ..) => println!("{}", i),
                                                                                                        (.., Some(s)) => println!("{}", s),
                                                                                                        _ => unreachable!(),
                                                                                                    })
                                                                                                })
                                                                                                    as Box<dyn for<'e> Fn(&'e [(Option<i32>, Option<String>)])>),
                                                                                                _ => unimplemented!(),
                                                                                            },
                                                                                        ))
                                                                                    })
                                                                                    .map(|(method, builtin)| match (method, builtin) {
                                                                                        (Some(_), ..) => {
                                                                                            unimplemented!("virtual dispatch")
                                                                                        }
                                                                                        (.., Some(b)) => b(&[pop((&mut stack, &mut sp))]),
                                                                                        _ => unreachable!(),
                                                                                    })
                                                                            }),
                                                                        _ => unreachable!(),
                                                                    })
                                                                })
                                                        })
                                                        .map(drop)
                                                        .unwrap_or_default()
                                                }
                                                e => eprintln!("unknown: {:02X}", e),
                                            })
                                        })
                                    })
                                })
                                .last()
                            })
                    })
                })
        })
        .map(drop)
        .unwrap_or_default()
}
