using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using NeoDatis.Odb.Test.Arraycollectionmap;
using NeoDatis.Odb.Test.Commit;
using NeoDatis.Odb;
using NeoDatis.Odb.Test.VO.Login;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Impl.Core.Layers.Layer2.Instance;
using System.IO;
using NeoDatis.Odb.Core.Oid;
using System.Collections;
using NeoDatis.Odb.Impl.Core.Oid;
using NeoDatis.Odb.Test.Query.Criteria;
using NeoDatis.Odb.Test.Enum;
using System.Reflection;
using NeoDatis.Odb.Core.Query.NQ;
using NeoDatis.Tool.Wrappers.Map;
using NeoDatis.Odb.Test.Insert;
using NeoDatis.Odb.Test.Delete;

namespace ManualTest
{
    class Program
    {
        delegate bool Match(Object o);
        static public void Test1()
        {
            String baseName = "test.neodatis";
            ODB odb = ODBFactory.Open(baseName);
            Function f = new Function("f1");
            odb.Store(f);
            odb.Close();

            odb = ODBFactory.Open(baseName);
            Objects<Function> functions = odb.GetObjects<Function>(new CriteriaQuery(Where.Equal("name","f1")));
            Console.WriteLine(functions.Count + " functions");
            odb.Close();
            Console.ReadLine();


        }

        static public void Test2()
        {
            String baseName = "t1.neodatis";
            ODB odb = ODBFactory.Open(baseName);
            Function f = new Function("f1");
            Profile profile = new Profile("profile1", f);
            User user = new User("user name", "email", profile);
            odb.Store(user);
            odb.Close();

            odb = ODBFactory.Open(baseName);
            Objects<User> users = odb.GetObjects<User>(new CriteriaQuery(Where.Equal("name", "user name")));
            Console.WriteLine(users.Count + " users");
            odb.Close();
            Console.ReadLine();


        }

        static public void TestUpdate()
        {
            String baseName = "t1.neodatis";
            File.Delete(baseName);
            ODB odb = ODBFactory.Open(baseName);
          
            Function f = new Function("f1");
            odb.Store(f);
            odb.Close();

            odb = ODBFactory.Open(baseName);
            Objects<Function> functions = odb.GetObjects<Function>();
            Console.WriteLine(functions.Count + " functions");
            f = functions.GetFirst();
            f.SetName("function 1");
            odb.Store(f);
            odb.Close();

            odb = ODBFactory.Open(baseName);
            functions = odb.GetObjects<Function>();
            Console.WriteLine(functions.Count + " functions after update");
            f = functions.GetFirst();
            Console.WriteLine(f.GetName());
            Console.ReadLine();


        }

        static public void Test3()
        {
            
            String baseName = "t2.neodatis";
            File.Delete(baseName);
            ODB odb = ODBFactory.Open(baseName);
            
            int size = 10000;
            DateTime now = DateTime.Now;
            for (int i = 0; i < size; i++)
            {
                Function f = new Function("f1 "+i);
                Profile profile = new Profile("profile1"+i, f);
                User user = new User("user name"+i, "email"+i, profile);
                odb.Store(user);
            }
            odb.Close();
            Console.WriteLine("time for insert = " + (DateTime.Now - now).Ticks/10000);
            
            odb = ODBFactory.Open(baseName);

            long t0 = DateTime.Now.Ticks;
            //Objects<User> users = odb.GetObjects<User>(new CriteriaQuery(Where.Like("name", "user name")));
            Objects<User> users = odb.GetObjects<User>(false);
            long t1 = DateTime.Now.Ticks;
            Console.WriteLine(users.Count + " users");
            while (users.HasNext())
            {
                users.Next();
            }
            long t2 = DateTime.Now.Ticks;
            Console.WriteLine("Get=" + (t1 - t0) / 10000 + "  - actual Get = " + (t2 - t1)/10000);
            odb.Close();
            Console.ReadLine();


        }

        static public void Test4()
        {

            String baseName = "t2.neodatis";
            File.Delete(baseName);
            ODB odb = ODBFactory.Open(baseName);
            Console.WriteLine("oi");
            int size = 10000;
            long now = DateTime.Now.Ticks;
            for (int i = 0; i < size; i++)
            {
                Function f = new Function("f1 " + i);
                odb.Store(f);
            }
            odb.Close();
            long t2 = DateTime.Now.Ticks;
            Console.WriteLine("time for insert = " + (DateTime.Now.Ticks - now)/10000);

            odb = ODBFactory.Open(baseName);

            //Objects<User> users = odb.GetObjects<User>(new CriteriaQuery(Where.Like("name", "user name")));
            Objects<Function> functions = odb.GetObjects<Function>();
            Console.WriteLine(functions.Count + " functions");
            Console.WriteLine(" Time = " + (DateTime.Now.Ticks - t2)/10000);
            odb.Close();
            Console.ReadLine();


        }


        static public void TestDictionnary()
        {
            int size = 100000;
            Dictionary<OID, Function> oids = new Dictionary<OID, Function>();
            long t0 = DateTime.Now.Ticks;
            long t = DateTime.Now.Ticks;
            for (int i = 0; i < size; i++)
            {
                OID oid = OIDFactory.BuildObjectOID(i);
                oids[oid] = new Function("function i");
                //Console.WriteLine(oid.GetHashCode());
                if (i % 10000 == 0)
                {
    
                    Console.WriteLine(i + " - " + (DateTime.Now.Ticks-t)/10000);
                    t = DateTime.Now.Ticks;
                }
            }
            for (int i = 0; i < size; i++)
            {
                Function f = null;
                oids.TryGetValue(OIDFactory.BuildObjectOID(i),out f);
                //Console.WriteLine(oid.GetHashCode());
                if (i % 10000 == 0)
                {

                    Console.WriteLine(i + " - " + (DateTime.Now.Ticks - t) / 10000);
                    t = DateTime.Now.Ticks;
                }
            }
            long t1 = DateTime.Now.Ticks;
            Console.WriteLine(" time is " + (t1 - t0)/10000);
            Console.ReadLine();
        }
        static public void TestHashTable()
        {
            Console.WriteLine("Test hash table");
            int size = 100000;
            IDictionary oids = new Hashtable();
            long t0 = DateTime.Now.Ticks;
            for (int i = 0; i < size; i++)
            {
                oids["test"+i] = new Function("function i");
                if (i % 10000 == 0)
                {
                    Console.WriteLine(i);
                }
            }
            for (int i = 0; i < size; i++)
            {
                Function f = (Function) oids["test" + i];
                if (i % 10000 == 0)
                {
                    Console.WriteLine(i);
                }
            }

            long t1 = DateTime.Now.Ticks;
            Console.WriteLine(" time is " + (t1 - t0)/10000);
            Console.ReadLine();
        }


        static void Main(string[] args)
        {
            Test3();
            Console.ReadLine();

            
        }
    }
    public class Query1 : SimpleNativeQuery
    {
        public bool Match(Function o)
        {
            return true;
        }
    }
}
