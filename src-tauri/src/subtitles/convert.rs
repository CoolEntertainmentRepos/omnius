/// Convert SRT subtitle format to WebVTT format
///
/// SRT format:
/// 1
/// 00:00:01,234 --> 00:00:05,678
/// Hello world
///
/// WebVTT format:
/// WEBVTT
///
/// 1
/// 00:00:01.234 --> 00:00:05.678
/// Hello world
pub fn srt_to_vtt(srt: &str) -> String {
    let mut vtt = String::from("WEBVTT\n\n");

    for line in srt.lines() {
        // Convert timestamp format: 00:00:01,234 --> 00:00:05,678
        // to: 00:00:01.234 --> 00:00:05.678
        if line.contains(" --> ") {
            // Replace comma with period for milliseconds
            let converted = line.replace(',', ".");
            vtt.push_str(&converted);
        } else {
            vtt.push_str(line);
        }
        vtt.push('\n');
    }

    vtt
}

/// Convert ASS/SSA subtitle format to WebVTT format
///
/// ASS format uses different timing format and has style info we strip
pub fn ass_to_vtt(ass: &str) -> String {
    let mut vtt = String::from("WEBVTT\n\n");
    let mut cue_number = 1;

    for line in ass.lines() {
        // ASS dialogue lines look like:
        // Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello world
        if line.starts_with("Dialogue:") {
            if let Some(dialogue) = parse_ass_dialogue(line) {
                vtt.push_str(&format!("{}\n", cue_number));
                vtt.push_str(&format!("{} --> {}\n", dialogue.start, dialogue.end));
                vtt.push_str(&dialogue.text);
                vtt.push_str("\n\n");
                cue_number += 1;
            }
        }
    }

    vtt
}

struct AssDialogue {
    start: String,
    end: String,
    text: String,
}

fn parse_ass_dialogue(line: &str) -> Option<AssDialogue> {
    // Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello world
    let parts: Vec<&str> = line.splitn(10, ',').collect();
    if parts.len() < 10 {
        return None;
    }

    let start = convert_ass_time(parts[1])?;
    let end = convert_ass_time(parts[2])?;

    // Text is the last part, may contain commas
    let text = parts[9]
        .replace("\\N", "\n")  // ASS newline
        .replace("\\n", "\n")  // ASS soft newline
        .replace("{\\i1}", "")  // Strip italic tags
        .replace("{\\i0}", "")
        .replace("{\\b1}", "")  // Strip bold tags
        .replace("{\\b0}", "");

    Some(AssDialogue { start, end, text })
}

fn convert_ass_time(time: &str) -> Option<String> {
    // ASS time: 0:00:01.00 (h:mm:ss.cs)
    // VTT time: 00:00:01.000 (hh:mm:ss.mmm)
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;

    let sec_parts: Vec<&str> = parts[2].split('.').collect();
    if sec_parts.len() != 2 {
        return None;
    }

    let seconds: u32 = sec_parts[0].parse().ok()?;
    let centiseconds: u32 = sec_parts[1].parse().ok()?;
    let milliseconds = centiseconds * 10;

    Some(format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, milliseconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srt_to_vtt() {
        let srt = "1\n00:00:01,234 --> 00:00:05,678\nHello world\n\n2\n00:00:06,000 --> 00:00:10,500\nGoodbye";
        let vtt = srt_to_vtt(srt);

        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:01.234 --> 00:00:05.678"));
        assert!(vtt.contains("Hello world"));
    }

    #[test]
    fn test_ass_time_conversion() {
        assert_eq!(convert_ass_time("0:00:01.00"), Some("00:00:01.000".to_string()));
        assert_eq!(convert_ass_time("1:23:45.67"), Some("01:23:45.670".to_string()));
    }
}
