using System;
using System.IO;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Pasta.Model;
using ProtoBuf;

namespace Pasta.Model.Test
{
    [TestClass]
    public class PastaLogTest
    {
        [TestMethod]
        public void TestMethod1()
        {
            using (var stream = new MemoryStream())
            {
                var pack = CreateRandPastaLog();
                pack.Application = "漢字付きのシリアライズがちゃんと行くか？";

                // シリアライズ
                Serializer.SerializeWithLengthPrefix<PastaLog>(stream, pack, PrefixStyle.Fixed32);

                // デシリアライズ
                stream.Position = 0;
                var unpack = Serializer.DeserializeWithLengthPrefix<PastaLog>(stream, PrefixStyle.Fixed32);
                Assert.AreEqual(pack.TimeID, unpack.TimeID);
                Assert.AreEqual(pack.Sender, unpack.Sender);
                Assert.AreEqual(pack.SenderGroup, unpack.SenderGroup);
                Assert.AreEqual(pack.Action, unpack.Action);
                Assert.AreEqual(pack.ActionGroup, unpack.ActionGroup);
                Assert.AreEqual(pack.ActionDescription, unpack.ActionDescription);
                Assert.AreEqual(pack.Target, unpack.Target);
                Assert.AreEqual(pack.TargetGroup, unpack.TargetGroup);
                Assert.AreEqual(pack.Item, unpack.Item);
                Assert.AreEqual(pack.Property, unpack.Property);
                Assert.AreEqual(pack.ResultText, unpack.ResultText);
                Assert.AreEqual(pack.ResultValue, unpack.ResultValue);
                Assert.AreEqual(pack.SourceID, unpack.SourceID);
            }
        }

        [TestMethod]
        public void TestMethod2()
        {

            using (var stream = new MemoryStream())
            {
                var p1 = CreateRandPastaLog();
                p1.Application = "漢字付きのシリアライズがちゃんと行くか？";
                Serializer.SerializeWithLengthPrefix<PastaLog>(stream, p1, PrefixStyle.Fixed32);

                var p2 = CreateRandPastaLog();
                p2.Application = "二つ目のシリアライズはちゃんと動くか？";
                Serializer.SerializeWithLengthPrefix<PastaLog>(stream, p2, PrefixStyle.Fixed32);

                stream.Position = 0;

                var u1 = Serializer.DeserializeWithLengthPrefix<PastaLog>(stream, PrefixStyle.Fixed32);
                Assert.AreEqual(p1.TimeID, u1.TimeID);
                Assert.AreEqual(p1.Application, u1.Application);

                var u2 = Serializer.DeserializeWithLengthPrefix<PastaLog>(stream, PrefixStyle.Fixed32);
                Assert.AreEqual(p2.TimeID, u2.TimeID);
                Assert.AreEqual(p2.Application, u2.Application);
            }

        }

        private static readonly Random rand = new Random();

        private static PastaLog CreateRandPastaLog()
        {
            var item = new PastaLog();
            item.SetUniqueTime();
            item.Application = RTEXT();
            item.Raw = RBYTE();
            item.Sender = RTEXT();
            item.SenderGroup = RTEXT();
            item.Action = RTEXT();
            item.ActionGroup = RTEXT();
            item.ActionDescription = RTEXT();
            item.Target = RTEXT();
            item.TargetGroup = RTEXT();
            item.Item = RTEXT();
            item.Property = RTEXT();
            item.ResultText = RTEXT();

            item.ResultValue = rand.Next();
            item.SourceID = rand.Next();

            return item;
        }

        private static string RTEXT()
        {
            return Guid.NewGuid().ToString();
        }
        private static byte[] RBYTE()
        {
            return Guid.NewGuid().ToByteArray();
        }
    }
}