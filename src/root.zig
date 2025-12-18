const std = @import("std");
const ContentId = @import("ContentId.zig");

test {
    std.testing.refAllDecls(ContentId);
}
