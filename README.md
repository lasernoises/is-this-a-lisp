### Is this a LISP?

```
[
  [let a 12]
  [let b [+ a 18]]
  [let c [* a b]]
  [let d [- c a]]

  d
]
```

I wanted to make a LISP. But I don't know what a LISP is. I heard it has something to do with lists.

#### File Extension

Since we're not sure whether this is a LISP you should use `.lisp?` as a file extension. That way if
someone who knows what a LISP is sees your file they can remove the `?` if it is indeed a LISP. If
not they can change it to `.not-lisp`.
