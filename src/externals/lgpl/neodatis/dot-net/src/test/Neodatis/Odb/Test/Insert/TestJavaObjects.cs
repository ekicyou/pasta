using System;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.Insert
{
	[TestFixture]
    public class TestJavaObjects : NeoDatis.Odb.Test.ODBTest
	{
		public static readonly string Name = "test.neodatis";

		/// <exception cref="System.Exception"></exception>
		public override void TearDown()
		{
			DeleteBase(Name);
		}

		public override void SetUp()
		{
			try
			{
				DeleteBase(Name);
			}
			catch (System.Exception e)
			{
                Console.WriteLine(e);
			}
		}

		/// <exception cref="System.Exception"></exception>
        [Test]
        public virtual void Test22StrignBuffer()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder("Ol√° chico");
			NeoDatis.Odb.ODB odb = Open(Name);
			odb.Store(buffer);
			odb.Close();
			odb = Open(Name);
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(System.Text.StringBuilder));
			odb.Close();
			System.Text.StringBuilder b2 = (System.Text.StringBuilder)l.GetFirst();
			AssertEquals(buffer.ToString(), b2.ToString());
		}

		/// <exception cref="System.Exception"></exception>
        [Test]
        public virtual void Test22JFrame()
		{
			Javax.Swing.JFrame frame = new Javax.Swing.JFrame("Ol\u00E1 chico");
			NeoDatis.Odb.ODB odb = Open(Name);
			odb.Store(frame);
			odb.Close();
			odb = Open(Name);
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(Javax.Swing.JFrame));
			odb.Close();
			Javax.Swing.JFrame frame2 = (Javax.Swing.JFrame)l.GetFirst();
			AssertEquals(frame.GetTitle(), frame2.GetTitle());
		}

		/// <summary>
		/// This junit does not work because of a problem? in URL: The hashcode of
		/// the 2 urls url1 & url2 are equal! even if theu point to different domains
		/// (having the same IP)
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
        [Test]
        public virtual void Test22URL()
		{
			Java.Net.URL url1 = new Java.Net.URL("http://wiki.neodatis.org");
			Java.Net.URL url2 = new Java.Net.URL("http://www.neodatis.org");
			object o1 = url1.GetContent();
			object o2 = url2.GetContent();
			int h1 = url1.GetHashCode();
			int h2 = url2.GetHashCode();
			Println(h1 + " - " + h2);
			Println(url1.GetHost() + " - " + url1.GetDefaultPort() + " - " + url1.GetFile() +
				 " - " + url1.GetRef());
			Println(url2.GetHost() + " - " + url2.GetDefaultPort() + " - " + url2.GetFile() +
				 " - " + url2.GetRef());
			Println(url1.GetHost().GetHashCode() + " - " + url1.GetDefaultPort() + " - " + url1
				.GetFile().GetHashCode() + " - " + url1.GetRef());
			Println(url2.GetHost().GetHashCode() + " - " + url2.GetDefaultPort() + " - " + url2
				.GetFile().GetHashCode() + " - " + url2.GetRef());
			NeoDatis.Odb.ODB odb = Open(Name);
			odb.Store(url1);
			odb.Store(url2);
			odb.Close();
			odb = Open(Name);
			NeoDatis.Odb.Objects l = odb.GetObjects(typeof(Java.Net.URL));
			odb.Close();
			if (testNewFeature)
			{
				AssertEquals("Same HashCode Problem", 2, l.Count);
			}
		}
	}
}
