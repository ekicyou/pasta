using NUnit.Framework;
using Test.Neodatis.Odb.Test.Enum;
namespace NeoDatis.Odb.Test.Enum
{
	[TestFixture]
    public class TestEnum : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
        [Test]
        public virtual void TestEnum1()
        {
            string baseName = GetBaseName();
            ODB odb = Open(baseName);
            ClassWithEnum e = new ClassWithEnum("enum1", ObjectType.Medium);
            odb.Store(e);
            odb.Close();
            odb = Open(baseName);
            Objects<ClassWithEnum> objects = odb.GetObjects<ClassWithEnum>();
            odb.Close();
            AssertEquals(1, objects.Count);
        }
        [Test]
        public virtual void TestEnumUpdate()
        {
            string baseName = GetBaseName();
            ODB odb = Open(baseName);
            ClassWithEnum e = new ClassWithEnum("enum1", ObjectType.Medium);
            odb.Store(e);
            odb.Close();
            odb = Open(baseName);
            Objects<ClassWithEnum> objects = odb.GetObjects<ClassWithEnum>();
            ClassWithEnum cwe = objects.GetFirst();
            cwe.SetObjectType(ObjectType.Small);
            odb.Store(cwe);
            odb.Close();
            odb = Open(baseName);
            objects = odb.GetObjects<ClassWithEnum>();
            AssertEquals(1, objects.Count);

            cwe = objects.GetFirst();
            odb.Close();
            AssertEquals(ObjectType.Small, cwe.GetObjectType());

        }
	}
}
