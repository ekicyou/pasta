using System;
using System.Collections.Generic;
using System.Linq;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace Test.Json
{
    [TestClass]
    public class JsonStudy
    {
        [TestMethod]
        public void TestMethod1()
        {
            var j = new JObject();
            {
                j.Add("Value1", new JValue(1));
                Assert.AreEqual(1, j.Value<int>("Value1"));
            }
            {
                var local = DateTime.Parse("2008-11-01T19:35:00.0000000Z");
                var utc = local.ToUniversalTime();
                j.Add("Value2", new JValue(utc));
                Assert.AreEqual(utc, j.Value<DateTime>("Value2"));
            }
        }
    }
}
