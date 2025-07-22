# Is this a LISP?

I don't know what a LISP is. So I decided to make one. But now I don't know whether what I've made
is a LISP.

## Examples

Every program is a single expression:

```nushell
"Hello, World!"
```

You can do basic math:

```nushell
[+ 2 [* 17 3]]
```

If you want to assign variables you need a `block`. The last entry in the `block` is the return
value.

```nushell
[block
    [let a 12]
    [let b [+ a 18]]
    [let c [* a b]]
    [let d [- c a]]

    d
]
```

Functions are just normal values:

```nushell
[block
    [let my_fn [fn [a b]
        # the content is like a block
        [let c [+ a b]]
        [* c a]
    ]]

    [my_fn 19.5 12]
]
```

The language is purely functional and features monadic I/O:

```nushell
[then
    [print_line "What is your name?"]
    [bind
        [read_line]
        [fn [name]
            [then
                [print_line "Hello,"]
                [print_line name]
            ]
        ]
    ]
]
```

There is a `do` notation to make this actually usable:

```nushell
[do
    [print_line "What is your name?"]

    # use is like let, but it internally uses bind to get the value the I/O evaluates to
    [use name [read_line]]

    # you can also use let in these, just like in blocks
    [let message "Hello,"]

    [print_line message]
    [print_line name]
]
```

## Installation

```sh
cargo install --git https://github.com/lasernoises/is-this-a-lisp.git
```

After you've installed it you can simply pass the path to your program to the executable:

```sh
is-this-a-lisp examples/hello_world.lisp?
```

## File Extension

Since we're not sure whether this is a LISP you should use `.lisp?` as a file extension. That way if
someone who knows what a LISP is sees your file they can remove the `?` if it is indeed a LISP. If
not they can change it to `.not-lisp`.

## Error Handling

This language features inscrutable error handling. Would you like scrutable error handling? I don't
think scrutable is even a word. You just made that up. So what do you even want from me?
