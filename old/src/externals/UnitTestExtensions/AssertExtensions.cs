using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using NUnit.Framework;

namespace NUnit.Framework
{
    public static class AssertExtensions
    {
        public static void Is(this decimal actual, decimal expected) { Assert.AreEqual(expected, actual); }
        public static void Is(this int actual, int expected) { Assert.AreEqual(expected, actual); }
        public static void Is(this long actual, long expected) { Assert.AreEqual(expected, actual); }
        public static void Is(this uint actual, uint expected) { Assert.AreEqual(expected, actual); }
        public static void Is(this ulong actual, ulong expected) { Assert.AreEqual(expected, actual); }
        public static void Is(this string actual, string expected) { Assert.AreEqual(expected, actual); }
        public static void Is(this object actual, object expected) { Assert.AreEqual(expected, actual); }

        public static void Is(this double actual, double expected, double delta) { Assert.AreEqual(expected, actual, delta); }
        public static void Is(this float actual, float expected, float delta) { Assert.AreEqual(expected, actual, delta); }


        public static void IsNot(this decimal actual, decimal expected) { Assert.AreNotEqual(expected, actual); }
        public static void IsNot(this int actual, int expected) { Assert.AreNotEqual(expected, actual); }
        public static void IsNot(this long actual, long expected) { Assert.AreNotEqual(expected, actual); }
        public static void IsNot(this uint actual, uint expected) { Assert.AreNotEqual(expected, actual); }
        public static void IsNot(this ulong actual, ulong expected) { Assert.AreNotEqual(expected, actual); }
        public static void IsNot(this string actual, string expected) { Assert.AreNotEqual(expected, actual); }
        public static void IsNot(this object actual, object expected) { Assert.AreNotEqual(expected, actual); }

    }
}
