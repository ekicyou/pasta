using System;
using System.Collections.Generic;
using System.Text;
using unit = NUnit;

namespace CSUtil.NUnit
{
    public static class ConsoleRunner
    {
        public static int Run(string[] args) { return unit.ConsoleRunner.Program.Run(args); }
    }
}