using Microsoft.VisualStudio.TestTools.UnitTesting;
using System;
using System.Collections.Generic;
using System.Linq;
using Util;
using System.Diagnostics;

namespace Test.Util
{
    [TestClass]
    public class GuidTest
    {
        private static readonly NLog.Logger logger = NLog.LogManager.GetCurrentClassLogger();

        [TestMethod]
        public void TestMethod1()
        {
            var items = Enumerable.Range(0, 32)
                .Select(a => Guid.NewGuid())
                .Select(a =>
                {
                    var bitText = a.ToBitString();
                    var base64 = a.ToBase64String();
                    var base64URL = a.ToBase64UrlString();
                    var guid = base64URL.Base64UrlToGuid();
                    Assert.AreEqual(guid, a);
                    return bitText + " | " + base64 + " | " + base64URL + " | " + (a == guid);
                });

            var text = string.Join("\r\n", items);
            Debug.WriteLine(text);
        }
    }
}
