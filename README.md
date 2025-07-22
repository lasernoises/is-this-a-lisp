### Is this a LISP?

```nushell
[block
  [let a 12]
  [let b [+ a 18]]
  [let c [* a b]]
  [let d [- c a]]

  d
]
```

I don't know what a LISP is. So I decided to make one. But now I don't know whether what I've made
is a LISP.

#### Installation

```sh
cargo install --git https://github.com/lasernoises/is-this-a-lisp.git
```

#### File Extension

Since we're not sure whether this is a LISP you should use `.lisp?` as a file extension. That way if
someone who knows what a LISP is sees your file they can remove the `?` if it is indeed a LISP. If
not they can change it to `.not-lisp`.

#### Error Handling

This lanugage features inscrutable error handling. Would you like scrutable error handling? I don't
think scrutable is even a word. You just made that up. So what do you even want from me?
