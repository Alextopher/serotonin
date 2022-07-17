# Design

The typical motivation for writing a toy language is to learn. In the spirit of learning, I will attempt to get into a lot of detail about the language's design.

## Motivation to use a stack-oriented Language

Compiling Brainfuck from a stack-oriented language is not a novel idea (TODO: create a list of similar languages). I _claim_ that I came up with the idea to use one independently. The general premise of the stack-oriented paradigm is:

**1.** The language operates on one or more stacks. In BFJOY, there is only one stack, and it is for data.

**2.** We use _postfix_ notation. Consider an expression written _infix_ `(1 + -2) * 3 - 6 == 0` in _postfix_ we would write `1 2 neg add 3 mul 6 sub 0 eq`. Simple expressions never need parentheses and in general, only when we insert control flow do we need to add annotations.

```joy
TODO: put an example if statement here
```

**3.** Stack-based algorithms operate by popping arguments off the top of the stack and pushing results back. We represent these manipulations using Stack Effect Diagrams

```( pops -- pushes )```

For example:

```text
# dup (a -- a a)

# drop (a -- )

# read ( -- a)

# print (a -- a)

# divmod (n d -- n%d n/d)
# I use spaces to seperate results

# 180 ( -- 180)
# I treat constants as functions that take no inputs and pushes their value on the stack
```

I see two significant challenges when writing Brainfuck, which can be simplified by thinking in a stack-oriented mindset.

### Quantum Data

Brainfuck data suffers from the [observer effect](https://en.wikipedia.org/wiki/Observer_effect_(physics)). In physics, the mere act of _observing_ a system can change it's state. In Brainfuck the same issue exists. Using data _necessarily_ requires that we destroy it.

Consider an addition routine:

```joy
Start: [ a b ]

  v inc a
[<+>-];
    ^ dec b

End: [ a+b 0 ]
```

This routine embodies what it means to follow a stack-orientated paradigm. We have destroyed our copies of `a` and `b` in exchange for their sum. If we wish to add `a` to `b` while maintaining a copy of `a`, we must add additional code to facilitate the copy.

```joy
Start: [ a b 0 ]

 v dec b
[->+<<+>]
   ^  ^
   inc two cells

End: [ a+b 0 a ]
```

We now have a copy of `a`, but it is not where it originated, which leads us to our next issue.

### Memory management

I think a good example is to consider some heavily code-golfed bf constants. For example, here is a [program](https://esolangs.org/wiki/Brainfuck_constants#180) that generates 180.

```text
>-[-<[+<]>>--]<
```

After the 4580 steps to reach program completion, the tape looks like this:

```text
[ 0 0 0 0 0 0 0 0 180 0 ]
                   ^
```

This routine needed to interact with every cell presented. Luckily we did not have data stored in those ten cells before we started! This issue comes up so often that _by convention_ Brainfuck routines shared online should never modify anything to the left of the starting data pointer. Algorithms will typically ask for some amount of zeros to the right. Here we needed ten temporary cells.

Also, consider if we had some data to the left before executing that code. Our magnificent 180 is _very_ far away. In Brainfuck we have access to an efficient routine I'll call "move until zero" `[<]`. So long as the data pointer sees nonzero cells, we move left. There is no equivalent "nice" way to "move until not zero." For most applications, spamming moves is the only reasonable way to cross a sea of zeros. For example, say we want to now add the 180 to the cell left of the starting position.

```text
>-[-<[+<]>>--]<[-<<<<<<<<+>>>>>>>>]

runtime: 8001
```

Having a sparse tape requires code that is both **uglier** and **slower**. We want to maintain a dense tape.

### Conclusion

We've identified two general _preferences_:

1. Keep your data on your right.

2. Maintain a dense tape.

If we upgrade these preferences from mere _suggestions_ to complete _invariants_, we end up with nothing more than a stack. Combining our stack with Brainfuck's sick desire to destroy all data transpiling from a stack-oriented programming language feels like a no-brainer.

## But why Joy?
