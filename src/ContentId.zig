const std = @import("std");
const ContentId = @This();

pub const FromReaderError = std.Io.Reader.StreamRemainingError || std.Io.Writer.Error;
const Hasher = std.hash.XxHash3;

id: u64,

pub fn init(id: u64) ContentId {
    return .{ .id = id };
}

pub fn fromSlice(slice: []const u8) ContentId {
    const id = Hasher.hash(0, slice);
    return ContentId.init(id);
}

pub fn fromReader(reader: *std.Io.Reader) FromReaderError!ContentId {
    var buffer: [1024]u8 = undefined;
    const hasher = Hasher.init(0);
    var hashing_writer = std.Io.Writer.Hashing(Hasher).initHasher(
        hasher,
        &buffer,
    );
    _ = try reader.streamRemaining(&hashing_writer.writer);
    try hashing_writer.writer.flush();
    const id = hashing_writer.hasher.final();
    return ContentId.init(id);
}

test init {
    const expected = ContentId{ .id = 0 };
    const actual = ContentId.init(0);
    try std.testing.expectEqual(expected, actual);
}

test fromSlice {
    const slice = [0]u8{};
    const expected = ContentId.init(3244421341483603138);
    const actual = ContentId.fromSlice(&slice);
    try std.testing.expectEqual(expected, actual);
}

test fromReader {
    const buffer = [0]u8{};
    var reader = std.Io.Reader.fixed(&buffer);
    const expected = ContentId.init(3244421341483603138);

    const actual = try ContentId.fromReader(&reader);

    try std.testing.expectEqual(expected, actual);
}
