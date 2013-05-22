using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Impl.Core.Layers.Layer1.Introspector;
using System;
namespace NeoDatis.Odb.Test.Intropector
{
	public class TestArray : NeoDatis.Odb.Test.ODBTest
	{
		public virtual void Test1()
		{
			string[] array = new string[] { "Ola", "chico" };
			AssertEquals(true, array.GetType().IsArray);
			AssertEquals("[Ljava.lang.String;", array.GetType().FullName);
			AssertEquals("java.lang.String", array.GetType().GetElementType().FullName);
		}

		public virtual void Test2()
		{
			int[] array = new int[] { 1, 2 };
			AssertEquals(true, array.GetType().IsArray);
			AssertEquals("[I", array.GetType().FullName);
			AssertEquals("int", array.GetType().GetElementType().FullName);
		}

		public virtual void Test3()
		{
			double[] array = new double[] { 1, 2 };
			AssertEquals(true, array.GetType().IsArray);
			AssertEquals("[D", array.GetType().FullName);
			AssertEquals("double", array.GetType().GetElementType().FullName);
		}

		/// <exception cref="System.TypeLoadException"></exception>
		/// <exception cref="Java.Lang.InstantiationException"></exception>
		/// <exception cref="System.MemberAccessException"></exception>
		public virtual void Test4()
		{
            Array o = System.Array.CreateInstance(typeof(int),5);
			o.SetValue(0, 1);
            o.SetValue(1, 2);
			AssertEquals(true, o.GetType().IsArray);
			AssertEquals("int", o.GetType().GetElementType().FullName);
			AssertEquals(1, OdbReflection.GetArrayElement(o, 0));
			AssertEquals(2, OdbReflection.GetArrayElement(o, 1));
            
		}
	}
}
