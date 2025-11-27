<official Rust pade doc>
Macros, A Methodical Introduction
This chapter will introduce Rust’s declarative Macro-By-Example system by explaining the system as a whole. It will do so by first going into the construct’s syntax and its key parts and then following it up with more general information that one should at least be aware of.

macro_rules!
With all that in mind, we can introduce macro_rules! itself. As noted previously, macro_rules! is itself a syntax extension, meaning it is technically not part of the Rust syntax. It uses the following forms:

macro_rules! $name {
    $rule0 ;
    $rule1 ;
    // …
    $ruleN ;
}
There must be at least one rule, and you can omit the semicolon after the last rule. You can use brackets([]), parentheses(()) or braces({}).

Each “rule” looks like the following:

    ($matcher) => {$expansion}
Like before, the types of parentheses used can be any kind, but parentheses around the matcher and braces around the expansion are somewhat conventional. The expansion part of a rule is also called its transcriber.

Note that the choice of the parentheses does not matter in regards to how the mbe macro may be invoked. In fact, function-like macros can be invoked with any kind of parentheses as well, but invocations with { .. } and ( ... );, notice the trailing semicolon, are special in that their expansion will always be parsed as an item.

If you are wondering, the macro_rules! invocation expands to… nothing. At least, nothing that appears in the AST; rather, it manipulates compiler-internal structures to register the mbe macro. As such, you can technically use macro_rules! in any position where an empty expansion is valid.

Matching
When a macro_rules! macro is invoked, the macro_rules! interpreter goes through the rules one by one, in declaration order. For each rule, it tries to match the contents of the input token tree against that rule’s matcher. A matcher must match the entirety of the input to be considered a match.

If the input matches the matcher, the invocation is replaced by the expansion; otherwise, the next rule is tried. If all rules fail to match, the expansion fails with an error.

The simplest example is of an empty matcher:

macro_rules! four {
    () => { 1 + 3 };
}
This matches if and only if the input is also empty (i.e. four!(), four![] or four!{}).

Note that the specific grouping tokens you use when you invoke the function-like macro are not matched, they are in fact not passed to the invocation at all. That is, you can invoke the above macro as four![] and it will still match. Only the contents of the input token tree are considered.

Matchers can also contain literal token trees, which must be matched exactly. This is done by simply writing the token trees normally. For example, to match the sequence 4 fn ['spang "whammo"] @_@, you would write:

macro_rules! gibberish {
    (4 fn ['spang "whammo"] @_@) => {...};
}
You can use any token tree that you can write.

Metavariables
Matchers can also contain captures. These allow input to be matched based on some general grammar category, with the result captured to a metavariable which can then be substituted into the output.

Captures are written as a dollar ($) followed by an identifier, a colon (:), and finally the kind of capture which is also called the fragment-specifier, which must be one of the following:

block: a block (i.e. a block of statements and/or an expression, surrounded by braces)
expr: an expression
ident: an identifier (this includes keywords)
item: an item, like a function, struct, module, impl, etc.
lifetime: a lifetime (e.g. 'foo, 'static, …)
literal: a literal (e.g. "Hello World!", 3.14, '🦀', …)
meta: a meta item; the things that go inside the #[...] and #![...] attributes
pat: a pattern
path: a path (e.g. foo, ::std::mem::replace, transmute::<_, int>, …)
stmt: a statement
tt: a single token tree
ty: a type
vis: a possible empty visibility qualifier (e.g. pub, pub(in crate), …)
For more in-depth description of the fragment specifiers, check out the Fragment Specifiers chapter.

For example, here is a macro_rules! macro which captures its input as an expression under the metavariable $e:

macro_rules! one_expression {
    ($e:expr) => {...};
}
These metavariables leverage the Rust compiler’s parser, ensuring that they are always “correct”. An expr metavariable will always capture a complete, valid expression for the version of Rust being compiled.

You can mix literal token trees and metavariables, within limits (explained in Metavariables and Expansion Redux).

To refer to a metavariable you simply write $name, as the type of the variable is already specified in the matcher. For example:

macro_rules! times_five {
    ($e:expr) => { 5 * $e };
}
Much like macro expansion, metavariables are substituted as complete AST nodes. This means that no matter what sequence of tokens is captured by $e, it will be interpreted as a single, complete expression.

You can also have multiple metavariables in a single matcher:

macro_rules! multiply_add {
    ($a:expr, $b:expr, $c:expr) => { $a * ($b + $c) };
}
And use them as often as you like in the expansion:

macro_rules! discard {
    ($e:expr) => {};
}
macro_rules! repeat {
    ($e:expr) => { $e; $e; $e; };
}
There is also a special metavariable called $crate which can be used to refer to the current crate.

Repetitions
Matchers can contain repetitions. These allow a sequence of tokens to be matched. These have the general form $ ( ... ) sep rep.

$ is a literal dollar token.

( ... ) is the paren-grouped matcher being repeated.

sep is an optional separator token. It may not be a delimiter or one of the repetition operators. Common examples are , and ;.

rep is the required repeat operator. Currently, this can be:

?: indicating at most one repetition
*: indicating zero or more repetitions
+: indicating one or more repetitions
Since ? represents at most one occurrence, it cannot be used with a separator.

Repetitions can contain any other valid matcher, including literal token trees, metavariables, and other repetitions allowing arbitrary nesting.

Repetitions use the same syntax in the expansion and repeated metavariables can only be accessed inside of repetitions in the expansion.

For example, below is a mbe macro which formats each element as a string. It matches zero or more comma-separated expressions and expands to an expression that constructs a vector.

macro_rules! vec_strs {
    (
        // Start a repetition:
        $(
            // Each repeat must contain an expression...
            $element:expr
        )
        // ...separated by commas...
        ,
        // ...zero or more times.
        *
    ) => {
        // Enclose the expansion in a block so that we can use
        // multiple statements.
        {
            let mut v = Vec::new();

            // Start a repetition:
            $(
                // Each repeat will contain the following statement, with
                // $element replaced with the corresponding expression.
                v.push(format!("{}", $element));
            )*

            v
        }
    };
}

fn main() {
    let s = vec_strs![1, "a", true, 3.14159f32];
    assert_eq!(s, &["1", "a", "true", "3.14159"]);
}
You can repeat multiple metavariables in a single repetition as long as all metavariables repeat equally often. So this invocation of the following macro works:

macro_rules! repeat_two {
    ($($i:ident)*, $($i2:ident)*) => {
        $( let $i: (); let $i2: (); )*
    }
}

repeat_two!( a b c d e f, u v w x y z );
But this does not:


repeat_two!( a b c d e f, x y z );
failing with the following error

error: meta-variable `i` repeats 6 times, but `i2` repeats 3 times
 --> src/main.rs:6:10
  |
6 |         $( let $i: (); let $i2: (); )*
  |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Metavariable Expressions
RFC: rfcs#1584
Tracking Issue: rust#83527
Feature: #![feature(macro_metavar_expr)]

Transcriber can contain what is called metavariable expressions. Metavariable expressions provide transcribers with information about metavariables that are otherwise not easily obtainable. With the exception of the $$ expression, these have the general form $ { op(...) }. Currently all metavariable expressions but $$ deal with repetitions.

The following expressions are available with ident being the name of a bound metavariable and depth being an integer literal:

${count(ident)}: The number of times $ident repeats in the inner-most repetition in total. This is equivalent to ${count(ident, 0)}.
${count(ident, depth)}: The number of times $ident repeats in the repetition at depth.
${index()}: The current repetition index of the inner-most repetition. This is equivalent to ${index(0)}.
${index(depth)}: The current index of the repetition at depth, counting outwards.
${len()}: The number of times the inner-most repetition will repeat for. This is equivalent to ${len(0)}.
${len(depth)}: The number of times the repetition at depth will repeat for, counting outwards.
${ignore(ident)}: Binds $ident for repetition, while expanding to nothing.
$$: Expands to a single $, effectively escaping the $ token so it won’t be transcribed.
 

For the complete grammar definition you may want to consult the Macros By Example chapter of the Rust reference.
<\official Rust pade doc>


<related Github issue>
Skip to content
Navigation Menu
rust-lang
rust

Type / to search
Code
Issues
5k+
Pull requests
918
Actions
Projects
9
Security
6
Insights
Tracking Issue for RFC 3086: macro metavariable expressions #83527
Open
Open
Tracking Issue for RFC 3086: macro metavariable expressions
#83527
@nikomatsakis
Description
nikomatsakis
opened on Mar 26, 2021 · edited by fmease
Contributor
This is a tracking issue for the RFC "3086" (rust-lang/rfcs#3086).
The feature gate for the issue is #![feature(macro_metavar_expr)].

About tracking issues
Tracking issues are used to record the overall progress of implementation.
They are also used as hubs connecting to other relevant issues, e.g., bugs or open design questions.
A tracking issue is however not meant for large scale discussion, questions, or bug reports about a feature.
Instead, open a dedicated issue for the specific matter and add the relevant feature gate label.

Steps

Implement the RFC (cc @markbt -- has there been work done here already?)

Adjust documentation (see instructions on rustc-dev-guide)

Stabilization PR (see instructions on rustc-dev-guide)
Unresolved questions and bugs

Figure out problems around hygiene
Implementation history
2022-02-25, [1/2] Implement macro meta-variable expressions
2022-03-11, [2/2] Implement macro meta-variable expressions
2022-03-12, Fix remaining meta-variable expression TODOs
2019-03-21, [macro-metavar-expr] Fix generated tokens hygiene
2022-04-07, Kickstart the inner usage of macro_metavar_expr
2022-04-07, [macro_metavar_expr] Add tests to ensure the feature requirement
2023-05-24, [RFC-3086] Restrict the parsing of count
2023-10-22, [RFC 3086] Attempt to try to resolve blocking concerns
Activity

nikomatsakis
added 
B-RFC-approved
Blocker: Approved by a merged RFC but not yet implemented.
 
T-lang
Relevant to the language team
 
C-tracking-issue
Category: An issue tracking the progress of sth. like the implementation of an RFC
 on Mar 26, 2021

nikomatsakis
mentioned this on Mar 26, 2021
RFC: Declarative macro metavariable expressions rfcs#3086

nikomatsakis
added 
F-macro_metavar_expr
`#![feature(macro_metavar_expr)]`
 on Mar 26, 2021
markbt
markbt commented on Mar 27, 2021
markbt
on Mar 27, 2021
There is a working prototype on the markbt/rust/metavariable_expressions branch. This needs feature gating, and there are a couple of TODOs to resolve, but it's otherwise in reasonable shape. I'm planning to work on it over the coming weeks.


nikomatsakis
mentioned this on Apr 7, 2021
Declarative macro repetition counts lang-team#57
markbt
markbt commented on Apr 7, 2021
markbt
on Apr 7, 2021
Update (2021-04-07)
I've not yet started work on productionizing the prototype on the markbt/rust/metavariable_expressions branch. I plan to start later this month, free time permitting.


PatchMixolydic
mentioned this on Apr 8, 2021
nested macros don't allow repetitions in binding patterns #35853

dalcde
added a commit that references this issue on Apr 12, 2021
Remove enum_dispatch dependency

ff7d720
markbt
markbt commented on May 9, 2021
markbt
on May 9, 2021
Update (2021-05-09)
I still haven't started on this yet as some stuff came up last month that prevented from having the time to work on it. It's still in my plan to work on it, and hopefully I'll have some time soon.

markbt
markbt commented on Jun 29, 2021
markbt
on Jun 29, 2021
Update (2021-06-29)
Still no progress, as I haven't had any spare time to work on this project. I'm still planning to work on it, and hopefully will get some time soon.

joshtriplett
joshtriplett commented on Jul 7, 2021
joshtriplett
on Jul 7, 2021
Member
@markbt If you don't expect to find the bandwidth in the near future, would you potentially be interested in seeking help in the form of another owner for this initiative? If you're still interested in driving this, that's fine, but if you'd like us to switch gears from pinging you to broadcasting that the project could use help, we'd be happy to do that.

markbt
markbt commented on Jul 9, 2021
markbt
on Jul 9, 2021
I'd be happy with any help if there's someone available. I still plan to work on it, but personal stuff is getting in the way at the moment. Sorry about that.

To recap: I have a working prototype on my branch at https://github.com/markbt/rust/tree/metavariable_expressions . The next steps are to rebase that onto the latest master, and then polish it up so that it's ready for inclusion. Then there's also the doc work to make sure the new feature is documented well. Help with any of this would be appreciated.


WorldSEnder
mentioned this on Sep 17, 2021
More hygenic version of styled_components futursolo/stylist-rs#48
c410-f3r
c410-f3r commented on Jan 29, 2022
c410-f3r
on Jan 29, 2022
Contributor
@markbt Are you still working on this feature? If not, then I can pick up from where you stopped

markbt
markbt commented on Jan 29, 2022
markbt
on Jan 29, 2022 via email
I haven't had a chance to work on it for a while. I'm still interested in it, so happy to help out if you're picking it up, it just had to take a back burner relative to some personal things.I have a branch with a prototype implementation on my github fork. It's likely very out of date, so will need rebasing up to something more recent. Or perhaps you can just use it for inspiration and start from scratch. Let me know if I can help at all, although I don't have a lot of free time at the moment.
c410-f3r
c410-f3r commented on Jan 30, 2022
c410-f3r
on Jan 30, 2022
Contributor
Thank you @markbt

I will take a look at the branch and then get in touch if any questions arise


c410-f3r
mentioned this on Feb 1, 2022
Implement macro meta-variable expressions #93545

c410-f3r
mentioned this in 3 pull requests on Mar 12, 2022
[1/2] Implement macro meta-variable expressions #94368
[2/2] Implement macro meta-variable expression #94833
Fix remaining meta-variable expression TODOs #94884

kennytm
mentioned this on Mar 13, 2022
macro_rules should expose accessors for total iteration count and current iteration number rfcs#407

matthiaskrgr
added a commit that references this issue on Mar 14, 2022
Rollup merge of rust-lang#94884 - c410-f3r:meta-take-2, r=petrochenkov

Verified
423b316
c410-f3r
c410-f3r commented on Mar 15, 2022
c410-f3r
on Mar 15, 2022
Contributor
The implementation has been merged so please try testing the feature as much as possible to find any potential bugs


c410-f3r
mentioned this on Mar 15, 2022
Add support for Meta-Variable Expressions rust-analyzer#11712
mark-i-m
mark-i-m commented on Mar 15, 2022
mark-i-m
on Mar 15, 2022
Contributor
Thanks @c410-f3r for all your hard work!

From my limited experience with this feature, I have some concerns about the design:

The depth arguments are confusing because some of the functions count up from the most nested depth and some count down from the outermost level.
Also, indexing does not universally start at 0, and an index of 1 means different things for different meta-variable expressions.
I find the semantics of count confusing and hard to keep track of. Which level is it counting? Does depth 1 mean count everything or count the 2nd nested loop? Does depth 1 mean count everything or count only the outermost loop? Maybe a name like total_count would be clearer? Maybe the depth option should not be optional? Maybe count should be removed, and people can just sum the output of length?
c410-f3r
c410-f3r commented on Mar 16, 2022
c410-f3r
on Mar 16, 2022 · edited by c410-f3r
Contributor
Thank you @mark-i-m for reviewing #93545 and for all your contributions to the compiler.

Hehehehe... You, me and @petrochenkov had trouble trying to understand this feature

The depth arguments are confusing because some of the functions count up from the most nested depth and some count down from the outermost level.
Indeed! It is much easier cognitively to keep everything with the same direction instead of having count going from outer-to-inner and index/length from inner-to-outer. Although I am not sure if semantic would be impacted.

Also, indexing does not universally start at 0, and an index of 1 means different things for different meta-variable expressions.
I find the semantics of count confusing and hard to keep track of. Which level is it counting? Does depth 1 mean count everything or count the 2nd nested loop? Does depth 1 mean count everything or count only the outermost loop? Maybe a name like total_count would be clearer? Maybe the depth option should not be optional? Maybe count should be removed, and people can just sum the output of length?
Yeap, someone that wrote ${count(foo)} will probably expect that ${count(foo, 1)} will return some value related to the amount of foos instead of the actual number of outer repetitions.

If repetitions are nested, then an optional depth parameter can be used to limit the number of nested repetitions that are counted. For example, a macro expansion like:

${count(x, 1)} ${count(x, 2)} 
c
o
u
n
t
(
x
,
3
)
( a 
(
b
( $x )* )* )*

The three values this expands to are the number of outer-most repetitions (the number of times a would be generated), the sum of the number of middle repetitions (the number of times b would be generated), and the total number of repetitions of $x.

https://github.com/markbt/rfcs/blob/macro_metavar_expr/text/0000-macro-metavar-expr.md#count

And as you said, throwing nested loops into the equation will alter indexing making understanding even harder. Not to mention mixing other meta-variable expressions like length 🙁.

#![feature(macro_metavar_expr)]

fn main() {
    macro_rules! mac {
        ( $( [ $( $i:ident )* ] )* ) => {{
            // ***** No loop *****
            
            println!("{}", ${count(i)}); // 5
            println!("{}", ${count(i, 0)}); // 2
            
            // Same as ${count(i)}
            //println!("{}", ${count(i, 1)});
            
            // Fobirdden. Index out of bounds
            //println!("{}", ${count(i, 2)});
            
            // ***** Outer-most loop *****
            
            $(
                println!("{}", ${count(i)}); // 3 and 2
                
                // Same as ${count(i)}
                //println!("{}", ${count(i, 0)});
                
                // Fobirdden. Index out of bounds
                //println!("{}", ${count(i, 1)});
            )*

            // ***** Outer-most and inner-most loops *****
            
            $(
                $(
                    ${ignore(i)}

                    // Forbidden. Can't be placed inside the inner-most repetition
                    //println!("{}", ${count(i)});
                )*
            )*
        }};
    }
    
    mac!([a b c] [d e]);
}
Maybe total_count and a mandatory depth can be nice modifications but I am also not sure about the removal of count (Useful for nested stuff). Overall, I think that users will have a hard time even with a good set of documentation.

As for myself, I learned to accept the things as they are currently defined in RFC, hehehehe 😁

c410-f3r
c410-f3r commented on Mar 16, 2022
c410-f3r
on Mar 16, 2022
Contributor
Any thoughts @markbt?

camsteffen
camsteffen commented on Mar 16, 2022
camsteffen
on Mar 16, 2022 · edited by camsteffen
Contributor
The depth arguments are confusing because some of the functions count up from the most nested depth and some count down from the outermost level.
+1. Being inconsistent here seems out of the question. I would vote for "distance from innermost", agreeing with the rationale in the RFC:

The meaning of the depth parameter in index and count originally counted inwards from the outer-most nesting. This was changed to count outwards from the inner-most nesting so that expressions can be copied to a different nesting depth without needing to change them.

Adding to this, having the depth parameters represent "distance from outermost" will lead to the following (unfortunate) user story:

I have a macro build_foo!. I want to refactor it to build_many_foos!, so I wrap the macro definition with $(...),*. Now all of the depth parameters are off by one so I increment them.

The only bad thing is that the English meaning of "depth" lends to "distance from outermost". But this is less important IMO.

We could support both by having negative numbers (e.g. ${index(-1)}) work as "distance from innermost", but that is probably not necessary. You'd also have to decide what to do with 0.

Maybe count should be removed, and people can just sum the output of length?

Yes this might be a "less is more" scenario. You could potentially get what you need using expressions of index and length, like ${length(1)+index(2)} or ${length(1)*length(2)}, but we'd have to support math within ${..}. Edit: this doesn't work

Position	Total
Current repetition	index	length
All repetitions		count
Has it been considered to make theses values bind-able within the macro pattern? This would take depth out of the picture completely since the variables are syntactically attached to a repetition. Inspired by ngFor in Angular.

macro_rules! mac {
    ($[i=index, l=length]($word:ident),*) => {
        $(println!("{}/{}: {}", $i, $l, stringify!($word));)*
    };
}
camsteffen
camsteffen commented on Mar 16, 2022
camsteffen
on Mar 16, 2022
Contributor
It also seems odd to me that index doesn't support an ident parameter but all the other functions do.

markbt
markbt commented on Mar 17, 2022
markbt
on Mar 17, 2022
First off, thanks to @c410-f3r for implementing this. Really excited to see it merged, and I'm going to try to find some time to try it out soon.

Thanks for all the feedback, too. I'm going to try to explain my thinking about this, as I spent some time thinking through these points when I was drafting the RFC. This was quite some time ago, so my memory is a little rusty (excuse the pun). Overall I think the RFC as it is currently is still the best way to do it, but perhaps we need to document it better to make it clearer how things work (especially count vs length).

It also seems odd to me that index doesn't support an ident parameter but all the other functions do.

Both index and length do not take ident parameters. This is because macro repetitions happen in lockstep, and so there is only one index and count for the current repetition, no matter which ident is involved. Contrast this with count where it does matter what ident it involved, as different idents can repeat different amounts in nested repetitions.

I find the semantics of count confusing and hard to keep track of. Which level is it counting? Does depth 1 mean count everything or count the 2nd nested loop? Does depth 1 mean count everything or count only the outermost loop? Maybe a name like total_count would be clearer? Maybe the depth option should not be optional? Maybe count should be removed, and people can just sum the output of length?

count was pretty much the reason I started this RFC in the first place. Part of the reason for metavar expressions is to avoid constructing large constant expressions that the compiler has to fold down, as these have a performance impacts and limitations with large numbers of repetitions which can be entirely avoided as the compiler already knows the answer.

It may be we need to improve documentation, as teaching things can sometimes be difficult. count can be thought of as "count the number of times this ident will appear". For most people there will be no need to use a depth parameter - the default of counting all depths will be what they want. The "limit to a particular number of depths" is likely only needed for special cases, when it will be worth learning the details of how it works. If we want to ensure values start at 0, I think ${count(indent, 0)} could be legitimately made to always evaluate to 1: i.e. how often does it repeat with no repetition, which is vacuously once, and shift everything down by one. We would need to distinguish "not provided" from "0" (perhaps effectively Option<usize>, where None means "all depths", although syntactically I think we would want to avoid having to write Some(2)).

We could support both by having negative numbers (e.g. ${index(-1)}) work as "distance from innermost", but that is probably not necessary. You'd also have to decide what to do with 0.

Negative numbers could work if that makes it more understandable. 0 would still mean "this depth", which is the default. In this case positive numbers >=1 would be meaningless, so the minus sign effectively becomes mandatory. Maybe another way would be to name it something different to make it clear it goes the other way (height, surround, level, ...).

The depth arguments are confusing because some of the functions count up from the most nested depth and some count down from the outermost level.

They all count from the current repetition level. count counts inwards, as it is concerned with the number of times the ident repeats within the repetition we're currently inside of. The default is "all of them", as that's the most common case. index and length count outwards as they are concerned with the place of the repetition we are currently inside of relative to the other repetitions it is itself part of. The default is the inner-most one, as, again, it's the most common case. I think this confusion perhaps lends some weight to using a different name for the depth parameters of index and length, but I don't have a good suggestion to hand.

Has it been considered to make theses values bind-able within the macro pattern?

That's really neat, but I do find it a little harder to understand (it's a bit un-Rust-lish). I'm also not sure how it would solve the count case or the ignore case. It's also quite a different design so we'd presumably need to spin through the RFC process again.

Yeap, someone that wrote ${count(foo)} will probably expect that ${count(foo, 1)} will return some value related to the amount of foos instead of the actual number of outer repetitions.

It is related to the number of foos - specifically it's the number of next-level repetitions that include foo, which might be different to the number of repetitions that include bar in the case of expressions like $( [ $( $( $foo ),* );* / $( $bar ),* ] )

camsteffen
camsteffen commented on Mar 17, 2022
camsteffen
on Mar 17, 2022
Contributor
Okay I now understand how count is categorically different from index and length. count is for inner repetitions and index and length are for outer repetitions, relative to the current expansion context. An ident is necessary for specifying an inner repetition since there could be multiple (and multiple within those etc.). But outer repetitions only need a relative depth. With this distinction in mind, it makes sense to me now that depth can point in two directions. (sorry for lagging) I would either rename depth to distance, or use a different word for "going outward" cases like level.

With regard to my alternate syntax idea, that would only work for the "outward looking" functions index and length. To that end, I am even more convinced that this would be good. It resolves the discrepancy with depth so that it now has its intuitive meaning. Having a different syntax for inward and outward looking values lends to a not-conflated mental model. To be clear, I am suggesting to keep count as is, but replace index and length as described earlier.

Another discrepancy I found in my understanding is that I assumed that count(x) would give the number of occurrences of $x in the macro input, but rather it is the number of times $x is expanded? This is usually but not always the same, and I think input count would be generally more useful and less surprising.

In any case, pointing out the inward/outward distinction upfront as it applies to the functions would resolve confusion. I think I assumed that all functions can be used in both directions (which doesn't make sense now).

camsteffen
camsteffen commented on Mar 17, 2022
camsteffen
on Mar 17, 2022 · edited by camsteffen
Contributor
For count, depth could be "outward distance from $x" rather than "inward distance from count(..)". I think that would be more consistent and intuitive.

Hope you don't mind one more alternate idea. Named repetitions. This allows for referencing repetitions that do not otherwise have an ident.

macro_rules! mac {
    ($h(hello)*) => {
        println!("hello {} times", ${count(h)});
    };
}
mac!(hello hello hello);
c410-f3r
c410-f3r commented on Mar 17, 2022
c410-f3r
on Mar 17, 2022 · edited by c410-f3r
Contributor
A little pseudo illustration. Hope it helps

meta

// Another example

#![feature(macro_metavar_expr)]

macro_rules! mac {
    ( $( [ $( ( $($i:ident)* ) )* ] )* ) => {
        [
            // ****** 6 `ident` repetitions *****
            //
            // 6 (a, b, c, d, e f)
            ${count(i)},
            
            // ****** 3 `[...]` repetitions *****
            //
            // 2 (a, b)
            // 4 (c d e f)
            // 0
            $( ${count(i)}, )*
            
            // ****** 5 `(...)` repetitions *****
            //
            // 2 (a, b)
            // 0
            // 1 (c)
            // 0
            // 3 (d e f)
            $( $( ${count(i)}, )* )*
        ]
    }
}

fn main() {
    let array = mac!([(a b) ()] [(c) () (d e f)] []);
    dbg!(array);
}
petrochenkov
petrochenkov commented on Mar 21, 2022
petrochenkov
on Mar 21, 2022
Contributor
Issue: literal tokens emitted by meta-variable expression need to have correct hygiene (SyntaxContext).

For that the span assigned to them needs to be processed by Marker (see fn transcribe for examples).

nikomatsakis
nikomatsakis commented on Mar 22, 2022
nikomatsakis
on Mar 22, 2022
Contributor
Author
@petrochenkov that sounds like a blocker for stabilization, right? (If so, I'll add to the OP)


c410-f3r
mentioned this on Mar 22, 2022
[macro-metavar-expr] Fix generated tokens hygiene #95188
mark-i-m
mark-i-m commented on Mar 22, 2022
mark-i-m
on Mar 22, 2022
Contributor
@nikomatsakis Yes, it is a blocker.


Dylan-DPC
added a commit that references this issue on Mar 22, 2022
Rollup merge of rust-lang#95188 - c410-f3r:aqui-vamos-nos, r=petroche…

Verified
7c38093
c410-f3r
c410-f3r commented on Apr 1, 2022
c410-f3r
on Apr 1, 2022
Contributor
If you guys don't mind, I would like to at least try stabilizing the only two things that weren't part of a discussion: ${ignore} and $$. They open a range of new possibilities, don't have the inner/outer dilemma and are IMO as useful as any counting method.


c410-f3r
mentioned this in 2 pull requests on Apr 7, 2022
Kickstart the inner usage of macro_metavar_expr #95761
[macro_metavar_expr] Add tests to ensure the feature requirement #95764

Dylan-DPC
added 6 commits that reference this issue on Apr 8, 2022
Rollup merge of rust-lang#95761 - c410-f3r:meta-var-stuff, r=petroche…

Verified
b9b64b1
Rollup merge of rust-lang#95764 - c410-f3r:metavar-test, r=petrochenkov

Verified
cfa8483
Rollup merge of rust-lang#95761 - c410-f3r:meta-var-stuff, r=petroche…

Verified
1f80881
Rollup merge of rust-lang#95764 - c410-f3r:metavar-test, r=petrochenkov

Verified
00c288c
Rollup merge of rust-lang#95764 - c410-f3r:metavar-test, r=petrochenkov

Verified
7328ae9
Rollup merge of rust-lang#95764 - c410-f3r:metavar-test, r=petrochenkov

Verified
0051301

c410-f3r
mentioned this on Apr 9, 2022
Stabilize $$ in Rust 1.63.0 #95860
c410-f3r
c410-f3r commented on Apr 9, 2022
c410-f3r
on Apr 9, 2022
Contributor
A stabilization attempt is available at #95860


clarfonthey
mentioned this on Apr 18, 2022
"meta-variable x repeats N times, but y repeats M times" error is confusing #96184

JohnTitor
added 2 commits that reference this issue on Jun 9, 2022
Rollup merge of rust-lang#95860 - c410-f3r:stabilize-meta, r=joshtrip…

Verified
41e40ec
Rollup merge of rust-lang#95860 - c410-f3r:stabilize-meta, r=joshtrip…

Verified
afa2edb

zjp-CN
mentioned this in 2 issues on Jun 29, 2022
关于count(ident) 是 count(ident, 0) 的简写的问题 zjp-CN/tlborm#7
${count(ident)} is not equivalent to ${count(ident, 0)} Veykril/tlborm#76
CAD97
CAD97 commented on Jul 8, 2022
CAD97
on Jul 8, 2022
Contributor
$$crate may be surprising: #99035


MasterPtato
mentioned this on Jul 10, 2022
Nested macros called with "constant" fragments can't capture said fragments as "constant" #99106

joshtriplett
added 
S-tracking-design-concerns
Status: There are blocking design concerns.
 on Jul 20, 2022
joshtriplett
joshtriplett commented on Jul 20, 2022
joshtriplett
on Jul 20, 2022
Member
Labeling as design concerns due to various discussions around semantics of $$ (e.g. $$crate), as well as questions about the bits not yet stabilized.

cybersoulK
cybersoulK commented on Oct 16, 2022
cybersoulK
on Oct 16, 2022
when is this stabilized? it's so important

c410-f3r
c410-f3r commented on Oct 16, 2022
c410-f3r
on Oct 16, 2022
Contributor
I will probably review the concerns and try stabilization again in the next months

CAD97
CAD97 commented on Oct 17, 2022
CAD97
on Oct 17, 2022
Contributor
The main blocking issue is determining the exact behavior we want out of $$crate and ensuring that's what's implemented. This is what reverted the first stabilization attempt, anyway.

This question is essentially whether $$crate should be equivalent to writing $crate in the crate where the macro which wrote $$crate or the crate which expanded that macro. #99035

Linking some relevant comments:

@petrochenkov dislikes $$ impacting hygiene Resolve $crate at the expansion-local crate #99445 (comment)
I don't model this as $$ impacting hygiene Resolve $crate at the expansion-local crate #99445 (comment)
How to replace the $crate pseudo-token with a binder expansion, which will make things clearer Forbid $crate in macro patterns #99447 (comment)
TL;DR $$ cannot just be stabilized as-is without addressing $crate somehow.

The second question is how the metafunction interface should work; I don't think there's been a decision here. TL;DR is it ${ignore(binder)} or ${ignore($binder)}? (What "kind" level do the metafunctions operate at? It would be nice if this same syntax could be used for other metafunctionality in a somewhat uniform way; the big one is perhaps eager macro expansion.)

With a decision there, ${ignore} could be stabilized without $$.


hasenbanck
added a commit that references this issue on Nov 19, 2022
Remove the need for once_cell.

952921e
stephanemagnenat
stephanemagnenat commented on Dec 16, 2022
stephanemagnenat
on Dec 16, 2022
Hello. Is there any progress on this issue? Maybe a min-stabilisation for index (is it possible to do it without solving other issues?)? As a user, I am dealing with a horrible super complex macro with 30 internal rules combining a push-down accumulator, a counter and quite a bunch of TT munching, to generate some description structures that are not that convoluted after all. Having the index variable would allow to get rid of a significant part of the complexity.

c410-f3r
c410-f3r commented on Dec 16, 2022
c410-f3r
on Dec 16, 2022 · edited by c410-f3r
Contributor
Yeah, index is very useful specially when dealing with tuples.

IMO, a partial-stabilization is unlikely to happen because of the unresolved concerns and I still could not find the time to work on this feature.

Due to low-traction and based on the following incomplete list of views that deviate from the original RFC, maybe it's worth considering creating an "amending" RFC?

Tracking Issue for RFC 3086: macro metavariable expressions #83527 (comment)
Tracking Issue for RFC 3086: macro metavariable expressions #83527 (comment)
https://internals.rust-lang.org/t/macro-meta-functions/16743?u=cad97
JarredAllen
JarredAllen commented on Feb 23, 2023
JarredAllen
on Feb 23, 2023
Contributor
Would it be possible to somehow stabilize $$ without stabilizing $$crate (I'm not sure if excluding $$crate is even technically possible, let alone practically feasible) and the rest of the proposed metavariable functionality? I'd like to be able to use $$ for defining macros inside the expansions of other macros (I have code right now that does this and it's quite ugly and difficult to read since I can't use $$).

Inspirateur
Inspirateur commented on Mar 7, 2023
Inspirateur
on Mar 7, 2023
this would be really nice 🥺

lowr
lowr commented on May 24, 2023
lowr
on May 24, 2023
Contributor
While fiddling around with ${count()}, I've noticed some unexpected behavior (at least to me). I'd like some clarification on whether they are intentional since they are not specified in the RFC unless I've missed.

${count(t,)} is accepted and interpreted as ${count(t, 0)}. I'd expected it to be rejected, and even if it were to be accepted, I'd have imagined it to be interpreted as ${count(t)}.

When the entire repetition (at some depth) is empty, depth parameter has no effect whatsoever.

macro_rules! foo {
    ($($t:ident)*) => { ${count(t, 4294967296)} }; 
}

macro_rules! bar {
    ( $( { $( [ $( ( $( $t:ident )* ) )* ] )* } )* ) => { ${count(t, 4294967296)} }
}

fn test() {
    foo!();            // successfully expands to 0
    bar!( { [] [] } ); // successfully expands to 0
}
(playground)

c410-f3r
c410-f3r commented on May 24, 2023
c410-f3r
on May 24, 2023
Contributor
While fiddling around with ${count()}, I've noticed some unexpected behavior (at least to me). I'd like some clarification on whether they are intentional since they are not specified in the RFC unless I've missed.

* `${count(t,)}` is accepted and interpreted as `${count(t, 0)}`. I'd expected it to be rejected, and even if it were to be accepted, I'd have imagined it to be interpreted as `${count(t)}`.

* When the entire repetition (at _some_ depth) is empty, depth parameter has no effect whatsoever.
  ```rust
  macro_rules! foo {
      ($($t:ident)*) => { ${count(t, 4294967296)} }; 
  }
  
  macro_rules! bar {
      ( $( { $( [ $( ( $( $t:ident )* ) )* ] )* } )* ) => { ${count(t, 4294967296)} }
  }
  
  fn test() {
      foo!();            // successfully expands to 0
      bar!( { [] [] } ); // successfully expands to 0
  }
  ```
  
  
      
        
      
  
        
      
  
      
    
  ([playground](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=56143f6e068dcddc6ef5c31fed1db791))
Yeah, ${count(t,)} probably shouldn't be allowed and out of bound indexes should panic IIRC.

Can you open an issue? I will try to fix the underlying problems.


lowr
mentioned this in 2 issues on May 24, 2023
${count(t,)} is interpreted as ${count(t, 0)} #111904
${count()} disregards depth parameter when captured repetition is empty #111905
lowr
lowr commented on May 24, 2023
lowr
on May 24, 2023
Contributor
@c410-f3r Thanks for checking it out quickly! Filed #111904 and #111905.


c410-f3r
mentioned this on May 25, 2023
[RFC-3086] Add a new concat metavar expr #111930

c410-f3r
mentioned this on Jun 21, 2023
[RFC-3086] Restrict the parsing of count #111908

cedricschwyter
added a commit that references this issue on Jul 1, 2023
chore:switch to rust nightly

Verified
021c4a7

cedricschwyter
mentioned this on Jul 1, 2023
chore:switch to rust nightly refactor:higher-order http method macro KyrillGobber/huehuehue#22

cedricschwyter
added a commit that references this issue on Jul 1, 2023
chore:switch to rust nightly

Verified
df0f69b
cybersoulK
cybersoulK commented on Aug 8, 2023
cybersoulK
on Aug 8, 2023
@CAD97 for real? $$ and $ignore were supposed to be stabilized for over a year now.
have you actually had a need for nested macros? i do, and this is a must requirement, why block stabilization because of $$crate, i never used it, and i can't se myself using $crate at all

cybersoulK
cybersoulK commented on Aug 8, 2023
cybersoulK
on Aug 8, 2023 · edited by cybersoulK
i am guessing the solution would be to merge $$ and $ignore ASAP,
and make usage of "$crate" behind an unstable feature until you decide what do to with $$crate

CAD97
CAD97 commented on Aug 8, 2023
CAD97
on Aug 8, 2023
Contributor
have you actually had a need for nested macros?

Yes, and I discovered the interesting behavior of $crate when writing a macro-expanded macro-expanded macro definition making use of $$ (at that time; it now uses stable-compatible techniques). I would absolutely and immediately benefit from the stabilization of both $$ and ${ignore} (or even better for my use cases, some kind of $:empty matcher).

i can't se myself using $crate at all [...] make usage of "$crate" behind an unstable feature

$crate is stable and very needed when defining resilient exported macros to be used by downstream crates. If your macro is #[macro_export]ed, you should be using $crate to refer to any item defined in your crate or upstream to you, such that downstream usage can't shadow your expected names and cause issues.

This becomes extremely necessary when using unsafe in the macro expansion, such that you can be absolutely sure that you completely control what code gets trusted and aren't exporting a macro making a soundness assumption that the user won't shadow any names that it uses.

until you decide what do to with $$crate

I originally claimed it wouldn't be possible to stabilize $$crate without

$$ and $ignore were supposed to be stabilized for over a year now.

$$ was in fact temporarily accepted for stabilization before it was backed out, but ${ignore} was not. ${ignore} was originally part of the proposed stabilization PR, but it was dropped before it was accepted due to design concerns.

why block stabilization

I don't actually have any power to directly block stabilization, let alone to revert an accepted stabilization on beta; if T-lang1 didn't agree with my concern about $$ they could just ignore me and stabilize it anyway. I was the one to post the revert PR, but I did so on request of T-lang.

Though tbf, I'm not blameless; I've proposed various resolutions and haven't followed up with an implementation. (This reminds me, I should ideally try my hand at properly implementing my desired fix for $crate semantics relatively soon, so it has a chance of landing for edition2024.) Time and motivation and hard to find concurrently, and no unpaid volunteer owes the project anything.

Footnotes
Technically, I am now a part of T-opsem (thus the [Member] tag), which is a subteam of T-lang. However, 1) I was not at the time of the revert (T-opsem didn't even exist), and 2) that membership does not confer T-lang membership or powers. Even if it did for some reason, I'd defer to the rest of T-lang on this. ↩

cybersoulK
cybersoulK commented on Aug 9, 2023
cybersoulK
on Aug 9, 2023 · edited by cybersoulK
@CAD97

$crate feels hacky to me. With macros 1.0, i force the user to import the entire scope of the macro, if it requires it.

I admit i am not an expert, but it might be worth to read my ideas for macro 2.0:

#39412 (comment)


c410-f3r
mentioned this on Oct 3, 2023
Rebalacing macro_metavar_expr to allow stabilization compiler-team#680
c410-f3r
c410-f3r commented on Oct 3, 2023
c410-f3r
on Oct 3, 2023
Contributor
Proposal
In hopes of making some progress, two approaches based on the feedback provided so far will be proposed here. Feel free to discuss alternatives that will lead to consensus if any of the following suggestions are not desired.

1. Innermost vs Outermost indexes
count uses outermost indices while length uses innermost indices and this inconsistency creates unnecessary confusion.

meta

To improve the situation, the order of all elements should start from the innermost index to the outermost index.

Mentions
Tracking Issue for RFC 3086: macro metavariable expressions #83527 (comment)
Tracking Issue for RFC 3086: macro metavariable expressions #83527 (comment)
Tracking Issue for RFC 3086: macro metavariable expressions #83527 (comment)
Tracking Issue for RFC 3086: macro metavariable expressions #83527 (comment)
Tracking Issue for RFC 3086: macro metavariable expressions #83527 (comment)
Tracking Issue for RFC 3086: macro metavariable expressions #83527 (comment)
2. $ prefix
Taking count as an example, should the syntax be count(some_metavariable) or count($some_metavariable)? The original RFC specified that metavariable expressions should refer metavariables without $ prefixes but there were some arguments in favour of $.

For unblocking purposes, the requirement of $ is being suggested. Such enforcement doesn't appear to incur a significant overhead besides the additional typing and interactions with $$ or multiple $$s shouldn't be a problem as long as the final expanded $ refers a metavariable.

Mentions
Stabilize $$ in Rust 1.63.0 #95860 (comment)
Stabilize $$ in Rust 1.63.0 #95860 (comment)
Stabilize $$ in Rust 1.63.0 #95860 (comment)
Stabilize $$ in Rust 1.63.0 #95860 (comment)
https://internals.rust-lang.org/t/macro-meta-functions/16743
https://rust-lang.zulipchat.com/#narrow/stream/213817-t-lang/topic/macro.20metafunctions.20vs.20eager.20expansion

c410-f3r
mentioned this on Oct 22, 2023
[RFC 3086] Attempt to try to resolve blocking concerns #117050
markbt
markbt commented on Oct 30, 2023
markbt
on Oct 30, 2023
It's nice to see this progressing. We recently found some more places where this would be useful, e.g. in constructing a tuple from a method that takes indexes using an expression like ( $( ${ignore($t)} row.get(${index()}), )* ).

1. Indexes
For ${index()} and ${length()} I think the inner-to-outer ordering is important, as it means you can move these expressions without having to re-index them.

For ${count(...)} I think either order is fine. I chose outer-to-inner as to my mind the count proceeds from outside to inside, but actually it doesn't matter. For reference I use this diagram to think about it (based on the definitions in the previous comment):

    .-- current
    |   .-- proposed
    |   |
N = 2   0  ----------.                     (default)
N = 1   1  -------.  |
N = 0   2  ----.  |  |
               v  v  v
$count($x, N)  [  (  $x $length(L)  )  ]
L = 0                <----------->         (default)
L = 1             <----------------->
L = 2           <--------------------->

This proposal switches the order of N, but it's fine for it to mean either way.

2. $ prefix
Again, I think this is fine. In fact, for $ignore it makes more sense, as we could later generalize it to more complex expressions with side-effects.

3. The $$crate problem.
Has anything been decided about this? A possible alternative would be to define ${crate()} which expands to the same thing as $crate but correctly in the case of recursive macros. It could be a warning to specify $$crate, and users should use $${crate()} instead, with ordinary $crate essentially becoming shorthand for ${crate()}.


petrochenkov
mentioned this on Nov 2, 2023
Tracking issue for concat_idents #29599
nikomatsakis
nikomatsakis commented on Nov 8, 2023
nikomatsakis
on Nov 8, 2023
Contributor
Author
Note from lang team: we discussed @c410-f3r's proposed changes in our meeting today and agree we should move forward, but please update the comment at top of the issue. More here.


sunshowers
mentioned this on Nov 29, 2023
[nexus] Make 'update_and_check' CTE explicitly request columns oxidecomputer/omicron#4572

bors
added a commit that references this issue on Dec 9, 2023
Auto merge of rust-lang#117050 - c410-f3r:here-we-go-again, r=petroch…

38de990

matthiaskrgr
added a commit that references this issue on Dec 12, 2023
Rollup merge of rust-lang#117050 - c410-f3r:here-we-go-again, r=petro…

Verified
13c0a20

bors
added a commit that references this issue on Dec 13, 2023
Auto merge of rust-lang#117050 - c410-f3r:here-we-go-again, r=petroch…

f651b43

github-actions
added a commit that references this issue on Dec 14, 2023
Auto merge of #117050 - c410-f3r:here-we-go-again, r=petrochenkov

56ee1bb
rslife
rslife commented on Dec 17, 2023
rslife
on Dec 17, 2023 · edited by rslife
Nesting ignores issue:

#![feature(macro_metavar_expr)]

struct Foo(i32);
struct Bar(i32);
struct Baz(i32);

macro_rules! mk_p {
    ($p_ty:ty $(, foo=$foo:ident)? $(, bar=$bar:ident)?) => {
        $(${ignore($foo)} mk_p!($p_ty, foo=$foo, foo_or_bar=foo_or_bar); )?
        $(${ignore($bar)} mk_p!($p_ty, bar=$bar, foo_or_bar=foo_or_bar); )?
    };
    ($p_ty:ty $(, foo=$foo:ident)? $(, bar=$bar:ident)? $(, baz=$baz:ident)? $(, foo_or_bar=$foo_or_bar:ident)?) => {
        impl $p_ty {
            fn p(&self) {
                $(
                    ${ignore($baz)}
                    eprintln!("baz={}", self.0);
                )?

                $(
                    ${ignore($foo_or_bar)}
                    let i = self.0 + 5;
                    $(
                        ${ignore($foo)}
                        eprintln!("foo={i}");
                    )?
                    $(
                        ${ignore($bar)}
                        eprintln!("bar={i}");
                    )?
                )?
            }
        }
    };
}

mk_p!(Foo, foo=foo);
mk_p!(Bar, bar=bar);
mk_p!(Baz, baz=baz);


fn main() {}
Error:

error: meta-variable `foo_or_bar` repeats 1 time, but `bar` repeats 0 times
  --> src/main.rs:20:18
   |
20 |                   $(
   |  __________________^
21 | |                     ${ignore($foo_or_bar)}
22 | |                     let i = self.0 + 5;
23 | |                     $(
...  |
30 | |                     )?
31 | |                 )?
   | |_________________^

error: meta-variable `foo_or_bar` repeats 1 time, but `foo` repeats 0 times
  --> src/main.rs:20:18
   |
20 |                   $(
   |  __________________^
21 | |                     ${ignore($foo_or_bar)}
22 | |                     let i = self.0 + 5;
23 | |                     $(
...  |
30 | |                     )?
31 | |                 )?
   | |_________________^


Is this a current limitation or intended behavior?

i18nsite
i18nsite commented on Jan 4, 2024
i18nsite
on Jan 4, 2024 · edited by i18nsite
the resolved
use ignore!

old
can $index support $index for xxx ? for example ${task.index}

image
c410-f3r
c410-f3r commented on Feb 9, 2024
c410-f3r
on Feb 9, 2024
Contributor
#117050 was merged ~2 months ago and no related issues have been created since then.

Can we finally proceed with the stabilization of everything but $$? Does anyone still have any kind of blocking concern?


tgross35
mentioned this on Mar 21, 2024
Moving macro metavar expressions forward wg-macros#4

c410-f3r
mentioned this on Mar 21, 2024
Stabilize count, ignore, index, and len (macro_metavar_expr) #122808
c410-f3r
c410-f3r commented on Mar 21, 2024
c410-f3r
on Mar 21, 2024
Contributor
A stabilization attempt is available at #122808


lnicola
added a commit that references this issue on Apr 7, 2024
Auto merge of #117050 - c410-f3r:here-we-go-again, r=petrochenkov

88d69d1

c410-f3r
mentioned this on Apr 21, 2024
Tracking Issue for macro_metavar_expr_concat #124225

A4-Tacks
mentioned this on Apr 25, 2024
New declarative macros, functions and fields not being recognized #91249

RalfJung
added a commit that references this issue on Apr 27, 2024
Auto merge of #117050 - c410-f3r:here-we-go-again, r=petrochenkov

1843d4d

junaadh
added a commit that references this issue on Aug 6, 2024
remove impl_opcode macro since repeation in metavariable expr is unst…

22d79e0

workingjubilee
mentioned this on Aug 7, 2024
alloc: add ToString specialization for &&str #128759
safinaskar
safinaskar commented on Oct 13, 2024
safinaskar
on Oct 13, 2024
Contributor
🚀 I found a way to evaluate concat_idents (and concat and few other built-in macros) before evaluating other macro, which takes concat_idents as an argument! I. e. I found a way to evaluate a!(concat_idents!(...)) such way, that concat_idents evaluates before a. Answer is crate https://crates.io/crates/with_builtin_macros !!! Thanks, @danielhenrymantilla ! In other words, with_builtin_macros is paste, but not only for concat_idents, but also for concat and some other macros.

Note: #[feature(macro_metavar_expr_concat)] is not complete solution (see below).

And in other words, with_builtin_macros allows one to achieve eager evaluation of macros in limited way.

Also, with_builtin_macros allows one to use concat_idents when defining new identifier.

Also, https://crates.io/crates/with_builtin_macros allows one to use concat_idents in stable Rust.

// (This code was not tested, may contain typos)

fn concat_idents!(a, b) () {} // Doesn't work

with_builtin_macros::with_eager_expansions! {
  fn #{ concat_idents!(a, b) } () {} // Works! Even on stable!
}

macro_rules! this_macro_accepts_ident {
  ($a:ident) => {}
}

// Doesn't work, because "this_macro_accepts_ident" evaluates before "concat_idents"
this_macro_accepts_ident!(concat_idents!(a, b));

with_builtin_macros::with_eager_expansions! {
  this_macro_accepts_ident!(#{ concat_idents!(a, b) }); // Works! Even on stable!
}

macro_rules! this_macro_accepts_literal {
  ($a:literal) => {}
}

// Doesn't work.
// Moreover, you cannot solve this problem using #[feature(macro_metavar_expr_concat)],
// because ${concat(...)} produces identifier, not string literal!!!
// Same applies to "paste"! "paste::paste!" deals with identifiers, not strings. So, with_builtin_macros is the only way!!!
this_macro_accepts_literal!(concat!("a", "b"));

with_builtin_macros::with_eager_expansions! {
  this_macro_accepts_literal!(#{ concat!("a", "b") }); // Works! Even on stable!
}

kaspar030
mentioned this on Mar 29
tracking: use of unstable features ariel-os/ariel-os#298

lcnr
mentioned this on Apr 14
remove reliance on a trait solver inference bug bevyengine/bevy#18840

github-merge-queue
added a commit that references this issue on Apr 14
remove reliance on a trait solver inference bug (#18840)

Verified
d7ec6a9

mockersf
added a commit that references this issue on Apr 14
remove reliance on a trait solver inference bug (#18840)

2a2ce6e

jf908
added a commit that references this issue on May 13
remove reliance on a trait solver inference bug (bevyengine#18840)

Verified
b62f5aa

brvtalcake
mentioned this on Jul 16
Add support for declarative macros v2 Daniel-Aaron-Bloom/eager2#16

ROMemories
mentioned this on Aug 29
refactor(env-utils): remove the dependency on konst ariel-os/ariel-os#1271

SimonSapin
added a commit that references this issue on Sep 26
Fix tracking issue number for feature(macro_attr)

Verified
497cf3c

SimonSapin
mentioned this in 2 pull requests on Sep 26
Fix tracking issue number for feature(macro_attr) SimonSapin/rust#2
Fix tracking issue number for feature(macro_attr) #147066

SimonSapin
added 2 commits that reference this issue on Sep 26
Fix tracking issue number for feature(macro_attr)

c474923
Fix tracking issue number for feature(macro_attr)

95c146a

dzmitry-lahoda
mentioned this on Sep 27
wait for https://github.com/rust-lang/rust/issues/83527 dzmitry-lahoda/enum-field#1

Zalathar
added a commit that references this issue on Sep 28
Rollup merge of rust-lang#147066 - SimonSapin:macro_attr-tracking, r=lqd

Verified
47c6f99

matthiaskrgr
added a commit that references this issue on Sep 28
Rollup merge of rust-lang#147066 - SimonSapin:macro_attr-tracking, r=lqd

Verified
194bd77

rust-timer
added a commit that references this issue on Sep 28
Unrolled build for #147066

Verified
c85e0a4
mwlon
mwlon commented 2 weeks ago
mwlon
2 weeks ago
Are there any recent updates to this? I would love to see it stabilized.

jhpratt
jhpratt commented 2 weeks ago
jhpratt
2 weeks ago
Member
@mwlon Someone needs to summarize the status and what's being stabilized. I previously volunteered for this, but I've been busy for so long I've not had the time to do so.

mwlon
mwlon commented 2 weeks ago
mwlon
2 weeks ago
@jhpratt Do you know how involved that might be? Wondering if it's something a potential first-time contributor like me could hope to accomplish in a day or two.

platonvin
Add a comment
new Comment
Markdown input: edit mode selected.
Write
Preview
Use Markdown to format your comment
Remember, contributions to this repository should follow its contributing guidelines, security policy and code of conduct.
Metadata
Assignees
No one assigned
Labels
B-RFC-approved
Blocker: Approved by a merged RFC but not yet implemented.
C-tracking-issue
Category: An issue tracking the progress of sth. like the implementation of an RFC
F-macro_metavar_expr
`#![feature(macro_metavar_expr)]`
S-tracking-design-concerns
Status: There are blocking design concerns.
T-lang
Relevant to the language team
Type
No type
Projects
No projects
Milestone
No milestone
Relationships
None yet
Development
No branches or pull requests
NotificationsCustomize
You're not receiving notifications from this thread.

Participants
@nikomatsakis
@joshtriplett
@stephanemagnenat
@safinaskar
@jhpratt
Issue actions
Footer
© 2025 GitHub, Inc.
Footer navigation
Terms
Privacy
Security
Status
Community
Docs
Contact
Manage cookies
Do not share my personal information
<\related Github issue>