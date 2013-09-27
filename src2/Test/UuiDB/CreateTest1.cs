using System;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using System.IO;
using UuiDB;

namespace Test.UuiDB
{

    [TestClass]
    public class CreateTest1
    {
        private static string TestDir { get; set; }

        [ClassInitialize]
        public static void ClassInitialize(TestContext context)
        {
            var guid = Guid.NewGuid();
            TestDir = Path.GetFullPath(guid.ToBase64UrlString());
            Directory.CreateDirectory(TestDir);
        }

        [ClassCleanup]
        public static void ClassCleanup()
        {
            Directory.Delete(TestDir, true);
        }


        [TestMethod]
        public void TestMethod1()
        {
        }
    }
}
