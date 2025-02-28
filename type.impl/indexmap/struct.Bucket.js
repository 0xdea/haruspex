(function() {
    var type_impls = Object.fromEntries([["indexmap",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Bucket%3CK,+V%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#155-184\">Source</a><a href=\"#impl-Bucket%3CK,+V%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;K, V&gt; <a class=\"struct\" href=\"indexmap/struct.Bucket.html\" title=\"struct indexmap::Bucket\">Bucket</a>&lt;K, V&gt;</h3></section></summary><div class=\"impl-items\"><section id=\"method.key_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#157-159\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.key_ref\" class=\"fn\">key_ref</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;K</a></h4></section><section id=\"method.value_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#160-162\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.value_ref\" class=\"fn\">value_ref</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;V</a></h4></section><section id=\"method.value_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#163-165\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.value_mut\" class=\"fn\">value_mut</a>(&amp;mut self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;mut V</a></h4></section><section id=\"method.key\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#166-168\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.key\" class=\"fn\">key</a>(self) -&gt; K</h4></section><section id=\"method.value\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#169-171\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.value\" class=\"fn\">value</a>(self) -&gt; V</h4></section><section id=\"method.key_value\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#172-174\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.key_value\" class=\"fn\">key_value</a>(self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.tuple.html\">(K, V)</a></h4></section><section id=\"method.refs\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#175-177\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.refs\" class=\"fn\">refs</a>(&amp;self) -&gt; (<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;K</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;V</a>)</h4></section><section id=\"method.ref_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#178-180\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.ref_mut\" class=\"fn\">ref_mut</a>(&amp;mut self) -&gt; (<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;K</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;mut V</a>)</h4></section><section id=\"method.muts\" class=\"method\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#181-183\">Source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"indexmap/struct.Bucket.html#tymethod.muts\" class=\"fn\">muts</a>(&amp;mut self) -&gt; (<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;mut K</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.85.0/std/primitive.reference.html\">&amp;mut V</a>)</h4></section></div></details>",0,"indexmap::set::Bucket"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-Bucket%3CK,+V%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#135-153\">Source</a><a href=\"#impl-Clone-for-Bucket%3CK,+V%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;K, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"indexmap/struct.Bucket.html\" title=\"struct indexmap::Bucket\">Bucket</a>&lt;K, V&gt;<div class=\"where\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    V: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#140-146\">Source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.0/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; Self</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.85.0/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#148-152\">Source</a><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.0/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, other: &amp;Self)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.85.0/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","indexmap::set::Bucket"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Bucket%3CK,+V%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#128\">Source</a><a href=\"#impl-Debug-for-Bucket%3CK,+V%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;K: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>, V: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"indexmap/struct.Bucket.html\" title=\"struct indexmap::Bucket\">Bucket</a>&lt;K, V&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#128\">Source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.85.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.85.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/1.85.0/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.85.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","indexmap::set::Bucket"],["<section id=\"impl-Copy-for-Bucket%3CK,+V%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/indexmap/lib.rs.html#128\">Source</a><a href=\"#impl-Copy-for-Bucket%3CK,+V%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;K: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>, V: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"indexmap/struct.Bucket.html\" title=\"struct indexmap::Bucket\">Bucket</a>&lt;K, V&gt;</h3></section>","Copy","indexmap::set::Bucket"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[8940]}