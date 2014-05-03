==========================
The biscuit parser library
==========================

:Author: MB and Christopher Diggins(original)
:Contact: mb2act@yahoo.co.jp 
:License: Distributed under the `Boost Software License`_, Version 1.0
:Version: 0.9xx

.. _biscuit: http://sourceforge.net/projects/p-stade/
.. _p-stade: http://p-stade.sourceforge.net/
.. _Christopher Diggins: http://www.cdiggins.com
.. _YARD: http://www.ootl.org/yard/
.. _C++ Template Metaprogramming: http://www.boost-consulting.com/metaprogramming-book.html
.. _Boost C++ Libraries: http://www.boost.org/
.. _Boost: http://www.boost.org/
.. _Boost Software License: http://www.boost.org/LICENSE_1_0.txt
.. _Boost.MPL: http://www.boost.org/libs/mpl/doc/
.. _Boost.Range: http://www.boost.org/libs/range/
.. _Boost.Spirit: http://spirit.sourceforge.net/
.. _Boost.Xpressive: http://boost-sandbox.sourceforge.net/libs/xpressive/doc/html/index.html
.. _Boost.Test: http://www.boost.org/libs/test/doc/

.. contents::

Preface
------------------------
I was looking for a light and unstrict xml parser.
`Boost.Spirit`_ and `Boost.Xpressive`_ were big and not good at compile-time performance.
I suspected that they were not *static*, but YARD_ written by `Christopher Diggins`_
was really static, small and fast.
In time, I noticed that the lazy instantiation of templates could allow us to write
recursive grammars, and I found that YARD_ and the finite state machine found at
`C++ Template Metaprogramming`_, could be binded. It was named biscuit_.

Introduction
------------
biscuit_ is an object-oriented recursive-descent parser generator framework
implemented using class templates. The templates allow us to author
Extended Backus-Normal Form (EBNF) in C++. Technical informations are available at YARD_.

A simple EBNF grammar snippet::

	group      ::= '(' expression ')'
	factor     ::= integer | group
	term       ::= factor (('*' factor) | ('/' factor))*
	expression ::= term (('+' term) | ('-' term))*

is approximated using biscuit_'s facilities as seen in this code snippet::

	D:\Application\biscuit_0_90_0\libs\doc\detail\introduction_0.ipp

Through the magic of the lazy *template instantiation*, they are the perfectly valid types.
The production rule **expression** is in fact a type that has a *static member function* **parse**.
As **parse** will be instantiated later by algorithms_, all you have to do is not to *define* but to *declare* a type::

	D:\Application\biscuit_0_90_0\libs\doc\detail\introduction_1.ipp

Quick Start
-----------

1. Get and install the latest release of the `Boost C++ Libraries`_. (biscuit_ uses only their headers.)
2. Include headers of biscuit_::

	D:\Application\biscuit_0_90_0\libs\doc\detail\quick_start_2.ipp

3. Define your own parser_ type::

	D:\Application\biscuit_0_90_0\libs\doc\detail\quick_start_3.ipp

4. Call algorithms_::

	D:\Application\biscuit_0_90_0\libs\doc\detail\quick_start_4.ipp

5. If you build test project, it is required to build `Boost.Test`_::

	bjam -sTOOLS=vc-7_1 --with-test install


Basic Concepts
--------------

Parser
^^^^^^

A parser_ is any type that has the *static member function*::

	D:\Application\biscuit_0_90_0\libs\doc\detail\basic_concept_parser.ipp

As a parser_ is a type, it can't have any runtime-state.
But you can pass any *UserState* object to algorithms_, and the object is
passed to the **parse**.

User State
^^^^^^^^^^

Any type.

Forward Range
^^^^^^^^^^^^^

A *Forward Range* is a concept similar to the STL *Container* concept.
A further document is available at `Boost.Range`_.

Semantic Action Class
^^^^^^^^^^^^^^^^^^^^^

A `semantic action class`_ can be any class of *Function Object*
that has the *member function*::

	D:\Application\biscuit_0_90_0\libs\doc\detail\basic_concept_semantic_action_type.ipp

Predefined Parsers
------------------

Some parser_ templates are predefined as a means for parser_ composition and embedding.

Primitives
^^^^^^^^^^

The table below lists EBNF and their equivalents in biscuit_.

	==================== ======================================================= ==================================================
	EBNF (or Perl)       biscuit                                                 Meaning
	==================== ======================================================= ==================================================
	.                    any                                                     any object
	-------------------- ------------------------------------------------------- --------------------------------------------------
	A | B                or_<A, B>                                               alternation of A and B
	-------------------- ------------------------------------------------------- --------------------------------------------------
	A B                  seq<A, B>                                               sequence of A and B
	-------------------- ------------------------------------------------------- --------------------------------------------------
	A*                   star<A>                                                 zero or more times, greedy
	-------------------- ------------------------------------------------------- --------------------------------------------------
	A+                   plus<A>                                                 one or more times, greedy
	-------------------- ------------------------------------------------------- --------------------------------------------------
	A?                   opt<A>                                                  zero or one time, greedy
	-------------------- ------------------------------------------------------- --------------------------------------------------
	A - B                minus<A, B>                                             match A, but the sub-match of A doesn't match B
	-------------------- ------------------------------------------------------- --------------------------------------------------
	A{n,m}               repeat<A, n, m>                                         between n and m times, greedy
	-------------------- ------------------------------------------------------- --------------------------------------------------
	A*? B                star_until<A, B>                                        zero or more As and B
	-------------------- ------------------------------------------------------- --------------------------------------------------
	"Diggins"            str<'D','i','g','g','i','n','s'>                        string
	-------------------- ------------------------------------------------------- --------------------------------------------------
	^                    begin                                                   beginning of sequence
	-------------------- ------------------------------------------------------- --------------------------------------------------
	$                    end                                                     end of sequence
	-------------------- ------------------------------------------------------- --------------------------------------------------
	\\n                  eol                                                     end of line
	-------------------- ------------------------------------------------------- --------------------------------------------------
	\\d                  digit                                                   a digit
	-------------------- ------------------------------------------------------- --------------------------------------------------
	\\D                  not_<digit>                                             not a digit
	-------------------- ------------------------------------------------------- --------------------------------------------------
	\\s                  space                                                   a space
	-------------------- ------------------------------------------------------- --------------------------------------------------
	\\S                  not_<space>                                             not a space
	-------------------- ------------------------------------------------------- --------------------------------------------------
	[0-9]                char_range<'0','9'>                                     characters in range '0' through '9'
	-------------------- ------------------------------------------------------- --------------------------------------------------
	[abc]                char_set<'a','b','c'>                                   characters 'a','b', or 'c'
	-------------------- ------------------------------------------------------- --------------------------------------------------
	[0-9abc]             or_< char_range<'0','9'>, char_set<'a','b','c'> >       characters 'a','b','c' or in range '0' though '9'
	-------------------- ------------------------------------------------------- --------------------------------------------------
	[^abc]               not_< char_set<'a','b','c'> >                           not characters 'a','b', or 'c'
	-------------------- ------------------------------------------------------- --------------------------------------------------
	(?=A)                before<A>                                               positive look-ahead assertion
	-------------------- ------------------------------------------------------- --------------------------------------------------
	(?!A)                not_< before<A> >                                       negative look-ahead assertion
	==================== ======================================================= ==================================================

YARD_ and biscuit_ have no back-tracking on star operations.
The maximum supported arity of parsers is now twenty.

Actor
^^^^^

actor_ is a parser_ that triggers a `semantic action class`_ object::

	D:\Application\biscuit_0_90_0\libs\doc\detail\predefined_parsers_actor.ipp

You can pass a `semantic action class`_ to actor_, but cannot pass a *function pointer*.
This trouble is fixed by grammar_ below.

Directives
^^^^^^^^^^
Directives_ are also parsers, some ports of `Boost.Spirit`_'s directives__.

	==================== ======================================================= ==================================================
	Boost.Spirit         biscuit                                                 Meaning
	==================== ======================================================= ==================================================
	lexeme_d[A]          impossible                                              turn off white space skipping
	-------------------- ------------------------------------------------------- --------------------------------------------------
	as_lower_d[A]        as_lower<A>                                             convert inputs to lower-case
	-------------------- ------------------------------------------------------- --------------------------------------------------
	no_actions[A]        no_actions<A>                                           all semantic actions not fire
	-------------------- ------------------------------------------------------- --------------------------------------------------
	???                  definitive_actions<A>                                   parse twice and suppress non-intended actions
	-------------------- ------------------------------------------------------- --------------------------------------------------
	longest_d[A|B]       longest<A, B>                                           choose the longest match
	-------------------- ------------------------------------------------------- --------------------------------------------------
	shortest_d[A|B]      shortest<A, B>                                          choose the shortest match
	-------------------- ------------------------------------------------------- --------------------------------------------------
	limit_d[A]           requires<A, PredicateClass>                             ensure the result of a parser is constrained
	-------------------- ------------------------------------------------------- --------------------------------------------------
	???                  transform<A, FunctorClass>                              convert inputs using functor
	==================== ======================================================= ==================================================

__ http://spirit.sourceforge.net/distrib/spirit_1_8_2/libs/spirit/doc/directives.html

Algorithms
----------

Algorithms_ of biscuit_ work with *Forward Range*. Bear in mind that
parsers don't know *value_type* of the range.
For instance, a parser_ **str** works fine if *value_type* of the
range is comparable with *char*.

match
^^^^^

**match** returns *true* if a parser_ run through the range; otherwise *false*::

	D:\Application\biscuit_0_90_0\libs\doc\detail\algorithms_match.ipp

search
^^^^^^

**search** returns the first sub matched **boost::iterator_range**; otherwise an *empty* range::

	D:\Application\biscuit_0_90_0\libs\doc\detail\algorithms_search.ipp

Ranges
------

filter_range
^^^^^^^^^^^^

**filter_range** is a filtered **boost::iterator_range** by a parser_::

	D:\Application\biscuit_0_90_0\libs\doc\detail\ranges_filter_range_0.ipp

There is no reason why chains of **filter_range** do not work::

	D:\Application\biscuit_0_90_0\libs\doc\detail\ranges_filter_range_1.ipp

match_results
^^^^^^^^^^^^^

**match_results** is a *Forward Range* of **boost::iterator_range**::

	D:\Application\biscuit_0_90_0\libs\doc\detail\ranges_match_results_0.ipp

Outputs::

	/* c comment no.1 */
	/* c comment no.2 */
	/* c comment no.3 */

Grammar
-------

As parsers are just types, they has no runtime-state. 
Nontype template argument is farely limited. 
If *value_type* of *Forward Range* is not *Integral Constants* like *char*, 
what can we do?
But `C++ Template Metaprogramming`_ says that *member function pointers* are available.
They can bind templates and objects.
`Boost.Spirit`_ makes a *expression templates object* from *expression templates objects*,
but you can make *expression type* from *expression templates* using biscuit_.

grammar_ binds parsers and objects::

	D:\Application\biscuit_0_90_0\libs\doc\detail\grammar_0.ipp

Now that biscuit_ has no limitation of *value_type* of *Forward Range* parsed. 
As **std::vector<std::string>** is a *Forward Range* of **std::string**, it works.
Keep in mind that *UserState* object is now your grammar object.

Full Example
^^^^^^^^^^^^

Here is a port of `Boost.Spirit`_'s calculator__::

	D:\Application\biscuit_0_90_0\libs\doc\detail\grammar_full_example.ipp

__ http://spirit.sourceforge.net/distrib/spirit_1_8_2/libs/spirit/doc/semantic_actions.html

**actor_** makes actor_ from the *member function pointer*.
Enjoy the simplicity, compile-time performance and smaller-size of the executable.


Debugger
--------

biscuit_ emulates `Boost.Spirit`_'s debugging__::

	D:\Application\biscuit_0_90_0\libs\doc\detail\debugging_0.ipp

__ http://spirit.sourceforge.net/distrib/spirit_1_8_2/libs/spirit/doc/debugging.html


debugger_ uses type-name of the first argument for outputs.
If your grammar_ is a *class template* like above,
type-name can be very long. So I think that you want to
define start_tag etc. 
Well, debugger_ automatically disappears on release-compile.

Outputs::

	1 + 2
	struct start_tag: "1+2"
	  struct expression_tag: "1+2"
	    struct term_tag: "1+2"
	      struct factor_tag: "1+2"
	        struct integer_tag: "1+2"
	push    1
	        /struct integer_tag: "+2"
	      /struct factor_tag: "+2"
	    /struct term_tag: "+2"
	    struct term_tag: "2"
	      struct factor_tag: "2"
	        struct integer_tag: "2"
	push    2
	        /struct integer_tag: ""
	      /struct factor_tag: ""
	    /struct term_tag: ""
	popped 1 and 2 from the stack. pushing 3 onto the stack.
	  /struct expression_tag: ""
	/struct start_tag: ""
	-------------------------
	Parsing succeeded
	result = 3
	-------------------------


Points of Interest
------------------

You can find the idea of *composing inlined algorithms* from `Boost.MPL TODO list`__.
YARD_ and biscuit_ seem to be the example of it. 
By the way, this article was the hopeless war, vs `Boost.Spirit`_. 
But don't you think another war will break out?

A snippet::

	D:\Application\biscuit_0_90_0\libs\doc\detail\cranberry_0.ipp

__ http://www.crystalclearsoftware.com/cgi-bin/boost_wiki/wiki.pl?MPL_TODO_List


References
----------

- `p-stade`_
- `Christopher Diggins`_
- YARD_
- `A Regular Expression Tokenizer using the YARD Parser`__
- `Parsing XML in C++ using the YARD Parser`__
- `C++ Template Metaprogramming`_
- `Boost C++ Libraries`_
- `Boost.MPL`_
- `Boost.Range`_
- `Boost.Spirit`_
- `Boost.Xpressive`_

__ http://www.codeproject.com/cpp/yard-tokenizer.asp
__ http://www.codeproject.com/cpp/yard-xml-parser.asp


Release Notes
-------------

- Fixed the name confusion between limit and **repeat**.
- Directives_ became first-class parsers.
- Added debugger_ parser_.

.. footer::

	This document was generated by Docutils_ from reStructuredText_ source and
	syntax-highlighted using biscuit_ itself.

.. _Docutils: http://docutils.sourceforge.net/index.html
.. _reStructuredText: http://docutils.sourceforge.net/rst.html
