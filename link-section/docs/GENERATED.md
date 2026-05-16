
# Crate Features

| Cargo feature | Description |
| --- | --- |
| `proc_macro` |  Crate feature `proc_macro` (enables the `#[section]` attribute shim). |

# Macro Attributes

<table><tr><th>Attribute</th><th>Description</th></tr>
<tr><td><code>aux(main = path::to::MAIN_SECTION)</code></td><td>

 Auxiliary sections are stored in a section near the main section. The
 aux path must be a valid reference to the main section.


</td></tr>
<tr><td><code>crate_path = ::path::to::link_section</code></td><td>

 Specify a custom crate path for the `link-section` crate. Used when
 re-exporting the section macro.


</td></tr>
<tr><td><code>macro_unique_name = my_unique_name_1234</code></td><td>

 Define a custom macro name for the submission macro. Note that this is
 required when not using the `proc_macro` feature and using `aux`.


</td></tr>
<tr><td><code>no_macro</code></td><td>

 Disable submission macro generation at the definition site.


</td></tr>
<tr><td><code>untyped | typed | mutable | movable | reference</code></td><td>

 The type of the section.


</td></tr>
</table>

# Defaults
