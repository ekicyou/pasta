//! Lua code generator for Pasta DSL.
//!
//! This module generates Lua source code from Pasta AST nodes.
//! Implements Requirements 1, 3a-3g for Lua code generation.

mod element_gen;
mod scope_gen;

use crate::config::LineEnding;
use crate::error::TranspileError;

use std::io::Write;

/// Lua code generator.
///
/// Generates Lua source code from Pasta AST nodes.
pub struct LuaCodeGenerator<'a, W: Write> {
    /// Output writer
    writer: &'a mut W,
    /// Current indentation level
    indent_level: usize,
    /// Line ending style
    line_ending: LineEnding,
}

impl<'a, W: Write> LuaCodeGenerator<'a, W> {
    /// Create a new Lua code generator.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            indent_level: 0,
            line_ending: LineEnding::default(),
        }
    }

    /// Create a new Lua code generator with specified line ending.
    pub fn with_line_ending(writer: &'a mut W, line_ending: LineEnding) -> Self {
        Self {
            writer,
            indent_level: 0,
            line_ending,
        }
    }

    /// Write indentation at current level.
    fn write_indent(&mut self) -> Result<(), TranspileError> {
        let indent = "    ".repeat(self.indent_level);
        write!(self.writer, "{}", indent)?;
        Ok(())
    }

    /// Write a line with current indentation.
    fn writeln(&mut self, s: &str) -> Result<(), TranspileError> {
        self.write_indent()?;
        write!(self.writer, "{}{}", s, self.line_ending.as_str())?;
        Ok(())
    }

    /// Write a blank line without indentation.
    fn write_blank_line(&mut self) -> Result<(), TranspileError> {
        write!(self.writer, "{}", self.line_ending.as_str())?;
        Ok(())
    }

    /// Write without indentation.
    fn write_raw(&mut self, s: &str) -> Result<(), TranspileError> {
        write!(self.writer, "{}", s)?;
        Ok(())
    }

    /// Increase indentation level.
    fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level.
    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Close a block: dedent, write `end`, and add a blank line.
    ///
    /// Common pattern used by actor, global scene, and local scene generators.
    fn end_block(&mut self) -> Result<(), TranspileError> {
        self.dedent();
        self.writeln("end")?;
        self.write_blank_line()?;
        Ok(())
    }

    /// Write the Lua header (require statement).
    pub fn write_header(&mut self) -> Result<(), TranspileError> {
        self.writeln("local PASTA = require \"pasta\"")?;
        self.writeln("local GLOBAL = require \"pasta.global\"")?;
        self.write_blank_line()?;
        Ok(())
    }
}
