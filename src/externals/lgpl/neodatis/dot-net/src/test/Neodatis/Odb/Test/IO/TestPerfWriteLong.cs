using System.IO;
using NUnit.Framework;
namespace NeoDatis.Odb.Test.IO
{
	[TestFixture]
    public class TestPerfWriteLong : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		[Test]
        public virtual void Test1()
		{
			int size = 100;
			System.IO.FileStream raf = new System.IO.FileStream(NeoDatis.Odb.Test.ODBTest.Directory
				 + "test1.neodatis", FileMode.OpenOrCreate);
			byte b = 1;
			byte[] bs = new byte[] { b, b, b, b, b, b, b, b };
			long l = 48987978;
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				raf.WriteByte(b);
                raf.WriteByte(b);
                raf.WriteByte(b);
                raf.WriteByte(b);
                raf.WriteByte(b);
                raf.WriteByte(b);
                raf.WriteByte(b);
                raf.WriteByte(b);
			}
			long t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long t3 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				raf.Write(bs);
			}
			long t4 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long t5 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				raf.WriteLong(l);
			}
			long t6 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			long t7 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				raf.Write(bs, 0, 8);
			}
			long t8 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			raf.Close();
			DeleteBase("test1.txt");
		}

		[Test]
        public virtual void Test2()
		{
			long l = 121654545;
			int size = 1000000;
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				byte[] bytes = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetByteArrayConverter
					().LongToByteArray(i);
			}
			long t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
		}

		public virtual void T2est3()
		{
			int size = 60000;
			int arraySize = 5000;
			byte[] bytes = null;
			long t1 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				bytes = new byte[arraySize];
			}
			long t2 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
			for (int i = 0; i < size; i++)
			{
				for (int j = 0; j < 800; j++)
				{
					bytes[j] = 0;
				}
			}
			long t3 = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
		}

		
	}
}
