const std = @import("std");
const easyversion = @import("easyversion");

pub fn main() !void {
    std.debug.print("All your {s} are belong to us.\n", .{"easyversion"});
}