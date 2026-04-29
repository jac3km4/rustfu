# Anm2 Animation Engine

This document outlines the features and data formats of the `Anm2` target animation engine and notes features that are currently missing in the Rust implementation.

## 1. General Structure

The root structure of an animation contains multiple components essential for parsing and rendering:

*   **Version Flags**: Determines features like atlas usage, local index usage, optimization, perfect hit testing, and transform index presence.
*   **Frame Rate**: The rate at which the animation should play.
*   **Index**: Metadata about the animation (see *Index Metadata* below).
*   **Texture**: Information about the texture maps the animation uses (including file name and CRC).
*   **Shapes**: Defines 2D quads representing regions in the texture atlas. Each shape has an ID, UV coordinates (top, left, bottom, right), pixel dimensions, and rendering offsets.
*   **Transform Table**: A shared pool of pre-calculated values for transformations (colors, rotations, translations) and actions. This table prevents redundant data in each frame.
*   **Sprites**: Definitions of individual sprite loops, containing frames and references to transformations or shapes.
*   **Imports**: External assets needed by the animation.

## 2. Frame Data Format

Frames describe how a sprite transforms over time. The binary format compresses the frame stream dynamically based on the highest integer value present in the data chunk.

The format defines three main streams for `FrameData`:
1.  **Bytes (`1`)**: Used when the maximum value is `< 255`.
2.  **Shorts (`2`)**: Used when the maximum value is `< 65535` (16-bit).
3.  **Ints (`4`)**: Used for any larger values (32-bit).

The Rust implementation currently parses these formats properly into the `FrameData` enum.

## 3. Transformations and Shapes

The engine uses a combination of basic transformations applied dynamically to shapes. A transformation frame specifies an ID corresponding to a particular blend of transformations.

The format explicitly maps different data descriptors (IDs) to specific specialized shape implementations:
*   `0`: Base shape (Identity)
*   `1` (`R`): Rotation and Skew
*   `2` (`T`): Translation
*   `3` (`RT`): Rotation + Translation
*   `4` (`A`): Color Addition (Additive Tinting)
*   `5` (`RA`): Rotation + Color Addition
*   `6` (`TA`): Translation + Color Addition
*   `7` (`RTA`): Rotation + Translation + Color Addition
*   `8` (`M`): Color Multiplication (Multiplicative Tinting)
*   `9` (`RM`): Rotation + Color Multiplication
*   `10` (`TM`): Translation + Color Multiplication
*   `11` (`RTM`): Rotation + Translation + Color Multiplication
*   `12` (`AM`): Color Addition + Color Multiplication
*   `13` (`RAM`): Rotation + Color Addition + Color Multiplication
*   `14` (`TAM`): Translation + Color Addition + Color Multiplication
*   `15` (`RTAM`): Rotation + Translation + Color Addition + Color Multiplication

### Missing Features in the Rust Implementation

The format also specifies higher IDs which correspond to compressed specialized transformations. These are currently **missing** from the Rust `frame_reader.rs` parsing logic:

*   **`49` (`CR`)**: Compressed Rotation
*   **`82` (`CT`)**: Compressed Translation
*   **`-77` (`CRT`)**: Compressed Rotation + Translation

Implementing these compressed variants will be necessary to fully support all animation files.

## 4. Sprite Definitions

Sprites represent an entity's animation logic. The engine categorizes them into different definitions (payloads) based on their playback requirements:

*   **Single (`SINGLE`)**: A single shape instance.
*   **Single No Action (`SINGLE_NO_ACTION`)**: A basic single shape with no action attachments.
*   **Single Frame (`SINGLE_FRAME`)**: A sprite consisting of one distinct frame spanning multiple units of duration.
*   **Frames (`FRAMES`)**: An indexed sprite containing a sequence of frames, lengths (durations), and corresponding actions to execute during playback.

Flags on the sprite also denote properties such as looping behavior and whether a custom string name is attached (e.g., `HAS_NAME`).

## 5. Actions

Frame sequences can embed actions that trigger game engine logic when an animation reaches a specific point. These include:

1.  `Go To Animation`
2.  `Go To Static Animation`
3.  `Run Script`
4.  `Go To Random Animation`
5.  `Hit`
6.  `Delete`
7.  `End`
8.  `Go To If Previous Animation`
9.  `Add Particle`
10. `Set Radius`

The Rust implementation currently models these properly in the `Action` enum.

## 6. Index Metadata

The index structure holds high-level directives on how the animation interacts with the rendering environment. The metadata flags dictate the presence of:

*   **Scale**: The default rendering scale factor.
*   **Render Radius**: Culling bounds or interaction radius.
*   **Hiding Parts**: Rules defining which visual parts of the animation are hidden when specific items are equipped.
*   **Parts Hidden By**: Rules defining which visual parts can hide other items.
*   **Extension**: Extra data files, specific height overrides per-part, and highlight colors.
*   **Perfect Hit Test**: Enables pixel-perfect cursor collision instead of bounding boxes.
*   **Flip Override**: Forces or prevents horizontal flipping.

These properties are parsed efficiently by the `AnimationIndex` struct in the target implementation.
