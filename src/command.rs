//! Command building and serialization for the Kitty graphics protocol

use crate::error::{Error, Result};
use crate::types::*;
use crate::{APC_END, APC_START, GRAPHICS_PREFIX, MAX_CHUNK_SIZE};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fmt;

/// Builder for constructing graphics protocol commands
#[derive(Debug, Clone, Default)]
pub struct CommandBuilder {
    /// Action to perform
    action: Option<Action>,
    /// Image format
    format: Option<ImageFormat>,
    /// Transmission medium
    medium: Option<TransmissionMedium>,
    /// Image width in pixels
    width: Option<u32>,
    /// Image height in pixels
    height: Option<u32>,
    /// Image ID (0-4294967295, must not be zero for some operations)
    image_id: Option<u32>,
    /// Image number (alternative to image_id)
    image_number: Option<u32>,
    /// Placement ID
    placement_id: Option<u32>,
    /// More data flag (0 = last chunk, 1 = more chunks)
    more_data: Option<bool>,
    /// Compression algorithm
    compression: Option<Compression>,
    /// Quiet mode (1 = suppress OK, 2 = suppress errors)
    quiet: Option<u8>,
    /// Source rectangle X offset
    source_x: Option<u32>,
    /// Source rectangle Y offset
    source_y: Option<u32>,
    /// Source rectangle width
    source_width: Option<u32>,
    /// Source rectangle height
    source_height: Option<u32>,
    /// Cell offset X (within current cell)
    cell_offset_x: Option<u32>,
    /// Cell offset Y (within current cell)
    cell_offset_y: Option<u32>,
    /// Number of columns to display
    columns: Option<u32>,
    /// Number of rows to display
    rows: Option<u32>,
    /// Z-index for stacking order
    z_index: Option<i32>,
    /// Cursor movement policy
    cursor_policy: Option<CursorPolicy>,
    /// Delete target
    delete_target: Option<DeleteTarget>,
    /// File path or shared memory name
    path: Option<String>,
    /// Data size for file/shared memory
    data_size: Option<usize>,
    /// Data offset for file/shared memory
    data_offset: Option<usize>,
    /// Unicode placeholder mode
    unicode_placeholder: Option<UnicodePlaceholder>,
    /// Parent image ID for relative placement
    parent_image_id: Option<u32>,
    /// Parent placement ID for relative placement
    parent_placement_id: Option<u32>,
    /// Horizontal offset for relative placement
    relative_h_offset: Option<i32>,
    /// Vertical offset for relative placement
    relative_v_offset: Option<i32>,
    /// Animation control
    animation_control: Option<AnimationControl>,
    /// Animation frame number (for various operations)
    frame_number: Option<u32>,
    /// Frame gap in milliseconds
    frame_gap: Option<i32>,
    /// Loop count (0 = ignored, 1 = infinite)
    loop_count: Option<u32>,
    /// Background color for frame (RGBA)
    background_color: Option<u32>,
    /// Reference frame for composition
    ref_frame: Option<u32>,
    /// Frame composition parameters
    composition: Option<FrameComposition>,
}

impl CommandBuilder {
    /// Create a new command builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the action
    pub fn action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    /// Set the image format
    pub fn format(mut self, format: ImageFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the transmission medium
    pub fn medium(mut self, medium: TransmissionMedium) -> Self {
        self.medium = Some(medium);
        self
    }

    /// Set the image dimensions (width, height) in pixels
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set the image ID
    pub fn image_id(mut self, id: u32) -> Self {
        self.image_id = Some(id);
        self
    }

    /// Set the image number (alternative to image ID)
    pub fn image_number(mut self, number: u32) -> Self {
        self.image_number = Some(number);
        self
    }

    /// Set the placement ID
    pub fn placement_id(mut self, id: u32) -> Self {
        self.placement_id = Some(id);
        self
    }

    /// Set the more data flag
    pub fn more_data(mut self, more: bool) -> Self {
        self.more_data = Some(more);
        self
    }

    /// Set compression
    pub fn compression(mut self, compression: Compression) -> Self {
        self.compression = Some(compression);
        self
    }

    /// Set quiet mode (1 = suppress OK, 2 = suppress errors)
    pub fn quiet(mut self, mode: u8) -> Self {
        self.quiet = Some(mode);
        self
    }

    /// Set the source rectangle (x, y, width, height)
    pub fn source_rect(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.source_x = Some(x);
        self.source_y = Some(y);
        self.source_width = Some(width);
        self.source_height = Some(height);
        self
    }

    /// Set cell offset (X, Y) within the current cell
    pub fn cell_offset(mut self, x: u32, y: u32) -> Self {
        self.cell_offset_x = Some(x);
        self.cell_offset_y = Some(y);
        self
    }

    /// Set display area in columns and rows
    pub fn display_area(mut self, columns: u32, rows: u32) -> Self {
        self.columns = Some(columns);
        self.rows = Some(rows);
        self
    }

    /// Set z-index
    pub fn z_index(mut self, z: i32) -> Self {
        self.z_index = Some(z);
        self
    }

    /// Set cursor policy
    pub fn cursor_policy(mut self, policy: CursorPolicy) -> Self {
        self.cursor_policy = Some(policy);
        self
    }

    /// Set delete target
    pub fn delete_target(mut self, target: DeleteTarget) -> Self {
        self.delete_target = Some(target);
        self
    }

    /// Set file path or shared memory name
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set data size and offset for file/shared memory
    pub fn data_range(mut self, size: usize, offset: usize) -> Self {
        self.data_size = Some(size);
        self.data_offset = Some(offset);
        self
    }

    /// Set unicode placeholder mode
    pub fn unicode_placeholder(mut self, columns: u16, rows: u16) -> Self {
        self.unicode_placeholder = Some(UnicodePlaceholder { columns, rows });
        self
    }

    /// Set parent for relative placement
    pub fn parent(mut self, image_id: u32, placement_id: u32) -> Self {
        self.parent_image_id = Some(image_id);
        self.parent_placement_id = Some(placement_id);
        self
    }

    /// Set relative offset for relative placement
    pub fn relative_offset(mut self, h: i32, v: i32) -> Self {
        self.relative_h_offset = Some(h);
        self.relative_v_offset = Some(v);
        self
    }

    /// Set animation control
    pub fn animation_control(mut self, control: AnimationControl) -> Self {
        self.animation_control = Some(control);
        self
    }

    /// Set frame number
    pub fn frame_number(mut self, frame: u32) -> Self {
        self.frame_number = Some(frame);
        self
    }

    /// Set frame gap in milliseconds (negative = gapless frame)
    pub fn frame_gap(mut self, gap_ms: i32) -> Self {
        self.frame_gap = Some(gap_ms);
        self
    }

    /// Set loop count (0 = ignored, 1 = infinite)
    pub fn loop_count(mut self, count: u32) -> Self {
        self.loop_count = Some(count);
        self
    }

    /// Set background color for frame (RGBA)
    pub fn background_color(mut self, color: u32) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set reference frame for composition
    pub fn ref_frame(mut self, frame: u32) -> Self {
        self.ref_frame = Some(frame);
        self
    }

    /// Set frame composition parameters
    pub fn composition(mut self, comp: FrameComposition) -> Self {
        self.composition = Some(comp);
        self
    }

    /// Build the command
    pub fn build(self) -> Command {
        Command { inner: self }
    }
}

/// A graphics protocol command ready for serialization
#[derive(Debug, Clone)]
pub struct Command {
    inner: CommandBuilder,
}

impl Command {
    /// Create a new command builder
    pub fn builder() -> CommandBuilder {
        CommandBuilder::new()
    }

    /// Build the control data string (key=value pairs)
    fn build_control_data(&self) -> String {
        let mut parts = Vec::new();

        // Action (a)
        if let Some(action) = &self.inner.action {
            parts.push(format!("a={action}"));
        }

        // Format (f)
        if let Some(format) = &self.inner.format {
            parts.push(format!("f={format}"));
        }

        // Transmission medium (t)
        if let Some(medium) = &self.inner.medium {
            parts.push(format!("t={medium}"));
        }

        // Image dimensions (s, v)
        if let Some(width) = self.inner.width {
            parts.push(format!("s={width}"));
        }
        if let Some(height) = self.inner.height {
            parts.push(format!("v={height}"));
        }

        // Image ID (i) or Image Number (I)
        if let Some(id) = self.inner.image_id {
            parts.push(format!("i={id}"));
        } else if let Some(num) = self.inner.image_number {
            parts.push(format!("I={num}"));
        }

        // Placement ID (p)
        if let Some(id) = self.inner.placement_id {
            parts.push(format!("p={id}"));
        }

        // More data flag (m)
        if let Some(more) = self.inner.more_data {
            parts.push(format!("m={}", if more { 1 } else { 0 }));
        }

        // Compression (o)
        if let Some(comp) = &self.inner.compression {
            parts.push(format!("o={comp}"));
        }

        // Quiet mode (q)
        if let Some(quiet) = self.inner.quiet {
            parts.push(format!("q={quiet}"));
        }

        // Source rectangle (x, y, w, h)
        if let Some(x) = self.inner.source_x {
            parts.push(format!("x={x}"));
        }
        if let Some(y) = self.inner.source_y {
            parts.push(format!("y={y}"));
        }
        if let Some(w) = self.inner.source_width {
            parts.push(format!("w={w}"));
        }
        if let Some(h) = self.inner.source_height {
            parts.push(format!("h={h}"));
        }

        // Cell offset (X, Y)
        if let Some(x) = self.inner.cell_offset_x {
            parts.push(format!("X={x}"));
        }
        if let Some(y) = self.inner.cell_offset_y {
            parts.push(format!("Y={y}"));
        }

        // Display area (c, r)
        if let Some(cols) = self.inner.columns {
            parts.push(format!("c={cols}"));
        }
        if let Some(rows) = self.inner.rows {
            parts.push(format!("r={rows}"));
        }

        // Z-index (z)
        if let Some(z) = self.inner.z_index {
            parts.push(format!("z={z}"));
        }

        // Cursor policy (C)
        if let Some(policy) = &self.inner.cursor_policy {
            if matches!(policy, CursorPolicy::NoMove) {
                parts.push(format!("C={policy}"));
            }
        }

        // Delete target (d)
        if let Some(target) = &self.inner.delete_target {
            parts.push(format!("d={}", target.code()));
        }

        // File path or shared memory name
        if let Some(_path) = &self.inner.path {
            // The path needs to be encoded in the payload, not control data
            // We'll handle this separately in serialize_with_path
        }

        // Data size (S) and offset (O)
        if let Some(size) = self.inner.data_size {
            parts.push(format!("S={size}"));
        }
        if let Some(offset) = self.inner.data_offset {
            parts.push(format!("O={offset}"));
        }

        // Unicode placeholder (U)
        if self.inner.unicode_placeholder.is_some() {
            parts.push("U=1".to_string());
        }

        // Parent for relative placement (P, Q)
        if let Some(id) = self.inner.parent_image_id {
            parts.push(format!("P={id}"));
        }
        if let Some(id) = self.inner.parent_placement_id {
            parts.push(format!("Q={id}"));
        }

        // Relative offset (H, V)
        if let Some(h) = self.inner.relative_h_offset {
            parts.push(format!("H={h}"));
        }
        if let Some(v) = self.inner.relative_v_offset {
            parts.push(format!("V={v}"));
        }

        // Animation control (s) - note: same letter as width, context matters
        // For animation control, this is set via action=a
        // The animation state is controlled by s=1/2/3
        if let Some(control) = &self.inner.animation_control {
            parts.push(format!("s={control}"));
        }

        // Frame number for various operations
        // For animation frame: c=frame number
        // For frame edit: r=frame number
        if let Some(frame) = self.inner.frame_number {
            // Context determines which key to use
            // For now, use 'c' for frame selection
            parts.push(format!("c={frame}"));
        }

        // Frame gap (z) - note: same letter as z-index
        // When action=f, z means frame gap
        if let Some(gap) = self.inner.frame_gap {
            if gap != 0 {
                parts.push(format!("z={gap}"));
            }
        }

        // Loop count (v) - note: same letter as height
        // When action=a, v means loop count
        if let Some(count) = self.inner.loop_count {
            if count > 0 {
                parts.push(format!("v={count}"));
            }
        }

        // Background color (Y)
        if let Some(color) = self.inner.background_color {
            parts.push(format!("Y={color}"));
        }

        // Reference frame (c) for frame composition
        // Already handled above as frame_number

        parts.join(",")
    }

    /// Serialize the command to an escape sequence string
    pub fn serialize(&self, data: &[u8]) -> Result<String> {
        let control = self.build_control_data();
        let encoded = STANDARD.encode(data);

        let mut result = Vec::new();

        // Start sequence
        result.extend_from_slice(APC_START);
        result.extend_from_slice(GRAPHICS_PREFIX.as_bytes());

        // Control data
        result.extend_from_slice(control.as_bytes());

        // Payload separator and payload
        result.push(b';');
        result.extend_from_slice(encoded.as_bytes());

        // End sequence
        result.extend_from_slice(APC_END);

        String::from_utf8(result).map_err(Error::from)
    }

    /// Serialize the command to bytes
    pub fn serialize_bytes(&self, data: &[u8]) -> Result<Vec<u8>> {
        let control = self.build_control_data();
        let encoded = STANDARD.encode(data);

        let mut result = Vec::new();

        result.extend_from_slice(APC_START);
        result.extend_from_slice(GRAPHICS_PREFIX.as_bytes());
        result.extend_from_slice(control.as_bytes());
        result.push(b';');
        result.extend_from_slice(encoded.as_bytes());
        result.extend_from_slice(APC_END);

        Ok(result)
    }

    /// Serialize command in chunks for large data
    /// Returns an iterator of escape sequences
    pub fn serialize_chunked(&self, data: &[u8]) -> Result<ChunkedSerializer> {
        // First, encode all data to base64
        let encoded = STANDARD.encode(data);

        // Calculate chunk size that's a multiple of 4
        let chunk_size = (MAX_CHUNK_SIZE / 4) * 4;

        Ok(ChunkedSerializer {
            control: self.build_control_data(),
            encoded,
            chunk_size,
            offset: 0,
            is_first: true,
        })
    }

    /// Serialize a command with a path (for file/shared memory transmission)
    pub fn serialize_with_path(&self) -> Result<String> {
        let control = self.build_control_data();
        let path = self.inner.path.as_ref().ok_or(Error::MissingField("path"))?;
        let encoded_path = STANDARD.encode(path.as_bytes());

        let mut result = Vec::new();

        result.extend_from_slice(APC_START);
        result.extend_from_slice(GRAPHICS_PREFIX.as_bytes());
        result.extend_from_slice(control.as_bytes());
        result.push(b';');
        result.extend_from_slice(encoded_path.as_bytes());
        result.extend_from_slice(APC_END);

        String::from_utf8(result).map_err(Error::from)
    }
}

/// Iterator for chunked serialization of large data
pub struct ChunkedSerializer {
    control: String,
    encoded: String,
    chunk_size: usize,
    offset: usize,
    is_first: bool,
}

impl ChunkedSerializer {
    /// Get the total number of chunks
    pub fn total_chunks(&self) -> usize {
        self.encoded.len().div_ceil(self.chunk_size)
    }

    /// Check if there are more chunks
    pub fn has_more(&self) -> bool {
        self.offset < self.encoded.len()
    }
}

impl Iterator for ChunkedSerializer {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.encoded.len() {
            return None;
        }

        let end = (self.offset + self.chunk_size).min(self.encoded.len());
        let chunk = &self.encoded[self.offset..end];
        let is_last = end >= self.encoded.len();

        let mut result = Vec::new();
        result.extend_from_slice(APC_START);
        result.extend_from_slice(GRAPHICS_PREFIX.as_bytes());

        if self.is_first {
            // First chunk includes all control data
            result.extend_from_slice(self.control.as_bytes());
            result.push(b',');
            self.is_first = false;
        }

        // m=1 for more data, m=0 for last chunk
        result.extend_from_slice(format!("m={}", if is_last { 0 } else { 1 }).as_bytes());
        result.push(b';');
        result.extend_from_slice(chunk.as_bytes());
        result.extend_from_slice(APC_END);

        self.offset = end;

        String::from_utf8(result).ok()
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command({})", self.build_control_data())
    }
}

/// Convenience functions for common operations
impl Command {
    /// Create a command to query protocol support
    pub fn query_support() -> Self {
        Self::builder()
            .action(Action::Query)
            .quiet(2)
            .build()
    }

    /// Create a command to transmit and display a PNG image
    pub fn transmit_png(data: &[u8]) -> Result<Vec<String>> {
        let cmd = Self::builder()
            .action(Action::TransmitAndDisplay)
            .format(ImageFormat::Png)
            .quiet(2)
            .build();

        let chunks: Vec<String> = cmd.serialize_chunked(data)?.collect();
        Ok(chunks)
    }

    /// Create a command to transmit and display raw RGBA data
    pub fn transmit_rgba(data: &[u8], width: u32, height: u32) -> Result<Vec<String>> {
        let expected_size = (width * height * 4) as usize;
        if data.len() != expected_size {
            return Err(Error::InvalidDimensions { width, height });
        }

        let cmd = Self::builder()
            .action(Action::TransmitAndDisplay)
            .format(ImageFormat::Rgba)
            .dimensions(width, height)
            .quiet(2)
            .build();

        let chunks: Vec<String> = cmd.serialize_chunked(data)?.collect();
        Ok(chunks)
    }

    /// Create a command to transmit and display raw RGB data
    pub fn transmit_rgb(data: &[u8], width: u32, height: u32) -> Result<Vec<String>> {
        let expected_size = (width * height * 3) as usize;
        if data.len() != expected_size {
            return Err(Error::InvalidDimensions { width, height });
        }

        let cmd = Self::builder()
            .action(Action::TransmitAndDisplay)
            .format(ImageFormat::Rgb)
            .dimensions(width, height)
            .quiet(2)
            .build();

        let chunks: Vec<String> = cmd.serialize_chunked(data)?.collect();
        Ok(chunks)
    }

    /// Create a command to delete all visible placements
    pub fn delete_all() -> Self {
        Self::builder()
            .action(Action::Delete)
            .delete_target(DeleteTarget::All)
            .build()
    }

    /// Create a command to delete an image by ID
    pub fn delete_by_id(image_id: u32) -> Self {
        Self::builder()
            .action(Action::Delete)
            .delete_target(DeleteTarget::ById { free_data: true })
            .image_id(image_id)
            .build()
    }

    /// Create a command to place a previously transmitted image
    pub fn place(image_id: u32, columns: u32, rows: u32) -> Self {
        Self::builder()
            .action(Action::Place)
            .image_id(image_id)
            .display_area(columns, rows)
            .build()
    }
}
