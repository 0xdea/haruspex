(function() {
    var type_impls = Object.fromEntries([["idalib_sys",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-Clone-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.85.1/src/core/clone.rs.html#174\">Source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: &amp;Self)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.85.1/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-Debug-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.1/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.85.1/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.85.1/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.1/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.85.1/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.85.1/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-ExternType-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-ExternType-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"cxx/extern_type/trait.ExternType.html\" title=\"trait cxx::extern_type::ExternType\">ExternType</a> for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3><div class=\"docblock\"><h4 id=\"safety\"><a class=\"doc-anchor\" href=\"#safety\">§</a>Safety</h4>\n<p>We assert that the namespace and type ID refer to a C++\ntype which is equivalent to this Rust type.</p>\n</div></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Id\" class=\"associatedtype trait-impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#associatedtype.Id\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"cxx/extern_type/trait.ExternType.html#associatedtype.Id\" class=\"associatedtype\">Id</a> = (c, __, u, l, o, n, g, l, o, n, g)</h4></section></summary><div class='docblock'>A type-level representation of the type’s C++ namespace and type name. <a href=\"cxx/extern_type/trait.ExternType.html#associatedtype.Id\">Read more</a></div></details><details class=\"toggle\" open><summary><section id=\"associatedtype.Kind\" class=\"associatedtype trait-impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#associatedtype.Kind\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"cxx/extern_type/trait.ExternType.html#associatedtype.Kind\" class=\"associatedtype\">Kind</a> = <a class=\"enum\" href=\"cxx/extern_type/kind/enum.Trivial.html\" title=\"enum cxx::extern_type::kind::Trivial\">Trivial</a></h4></section></summary><div class='docblock'>Either <a href=\"cxx/extern_type/kind/enum.Opaque.html\" title=\"enum cxx::extern_type::kind::Opaque\"><code>cxx::kind::Opaque</code></a> or <a href=\"cxx/extern_type/kind/enum.Trivial.html\" title=\"enum cxx::extern_type::kind::Trivial\"><code>cxx::kind::Trivial</code></a>. <a href=\"cxx/extern_type/trait.ExternType.html#associatedtype.Kind\">Read more</a></div></details></div></details>","ExternType","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-From%3Cu64%3E-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-From%3Cu64%3E-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.1/std/primitive.u64.html\">u64</a>&gt; for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#method.from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.1/core/convert/trait.From.html#tymethod.from\" class=\"fn\">from</a>(val: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.1/std/primitive.u64.html\">u64</a>) -&gt; <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h4></section></summary><div class='docblock'>Converts to this type from the input type.</div></details></div></details>","From<u64>","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Hash-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-Hash-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a> for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.hash\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#method.hash\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hash.html#tymethod.hash\" class=\"fn\">hash</a>&lt;__H&gt;(&amp;self, state: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.1/std/primitive.reference.html\">&amp;mut __H</a>)<div class=\"where\">where\n    __H: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\">Hasher</a>,</div></h4></section></summary><div class='docblock'>Feeds this value into the given <a href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\"><code>Hasher</code></a>. <a href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hash.html#tymethod.hash\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.hash_slice\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.3.0\">1.3.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.85.1/src/core/hash/mod.rs.html#235-237\">Source</a></span><a href=\"#method.hash_slice\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hash.html#method.hash_slice\" class=\"fn\">hash_slice</a>&lt;H&gt;(data: &amp;[Self], state: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.1/std/primitive.reference.html\">&amp;mut H</a>)<div class=\"where\">where\n    H: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\">Hasher</a>,\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class='docblock'>Feeds a slice of this type into the given <a href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\"><code>Hasher</code></a>. <a href=\"https://doc.rust-lang.org/1.85.1/core/hash/trait.Hash.html#method.hash_slice\">Read more</a></div></details></div></details>","Hash","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-PartialEq-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.1/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;<a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.1/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>self</code> and <code>other</code> values to be equal, and is used by <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.85.1/src/core/cmp.rs.html#261\">Source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.1/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.1/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.1/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>!=</code>. The default implementation is almost always sufficient,\nand should not be overridden without very good reason.</div></details></div></details>","PartialEq","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"],["<section id=\"impl-Copy-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-Copy-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3></section>","Copy","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"],["<section id=\"impl-Eq-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-Eq-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3></section>","Eq","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"],["<section id=\"impl-StructuralPartialEq-for-c_ulonglong\" class=\"impl\"><a class=\"src rightside\" href=\"src/autocxx/lib.rs.html#449-453\">Source</a><a href=\"#impl-StructuralPartialEq-for-c_ulonglong\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.StructuralPartialEq.html\" title=\"trait core::marker::StructuralPartialEq\">StructuralPartialEq</a> for <a class=\"struct\" href=\"autocxx/struct.c_ulonglong.html\" title=\"struct autocxx::c_ulonglong\">c_ulonglong</a></h3></section>","StructuralPartialEq","idalib_sys::ffi::bindgen::root::ea_t","idalib_sys::ffi::bindgen::root::uval_t"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[15337]}