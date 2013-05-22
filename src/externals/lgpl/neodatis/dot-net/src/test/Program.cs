using System;
using System.Collections.Generic;
using System.Text;
using NeoDatis.Odb;
using NeoDatis.Tool;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;

namespace Test
{
    class Program
    {
        public static void test1()
        {
            //OdbConfiguration.SetDebugEnabled(true);
            OdbConfiguration.SetReconnectObjectsToSession(false);
           
            try
            {
                string file = "Test.NeoDatis";
                IOUtil.DeleteFile(file);
                ODB odb = ODBFactory.Open(file);
                OID oid = odb.Store(new Function("f1"));
                odb.Close();
                Console.WriteLine("Write Done!");

                odb = ODBFactory.Open(file);
                Objects<Function> functions = odb.GetObjects<Function>();
                Console.WriteLine(" Number of functions = " + functions.Count);
                Function f = (Function) odb.GetObjectFromId(oid);
                Console.WriteLine(f.ToString());
                odb.Close();
            }
            catch (Exception e)
            {
                Console.WriteLine(e);
            }
            Console.ReadLine();
        }

        public static void test2()
        {
            //OdbConfiguration.SetDebugEnabled(true);
            OdbConfiguration.SetReconnectObjectsToSession(false);

            try
            {
                int size = 1000;
                string file = "Test.NeoDatis";
                IOUtil.DeleteFile(file);
                ODB odb = ODBFactory.Open(file);
                for (int i = 0; i < size; i++)
                {
                    OID oid = odb.Store(new Function("function " + i));
                }
                odb.Close();

                odb = ODBFactory.Open(file);
                Objects<Function> functions = odb.GetObjects<Function>();
                Console.WriteLine(" Number of functions = " + functions.Count);
                
                odb.Close();
            }
            catch (Exception e)
            {
                Console.WriteLine(e);
            }
            Console.ReadLine();
        }

        public static void test4()
        {
            //OdbConfiguration.SetDebugEnabled(true);
            OdbConfiguration.SetReconnectObjectsToSession(false);

            try
            {
                int size = 1000;
                string file = "Test.NeoDatis";
                Console.WriteLine("Oi");
                IOUtil.DeleteFile(file);
                ODB odb = ODBFactory.Open(file);
                for (int i = 0; i < size; i++)
                {
                    OID oid = odb.Store(new Function("function " + i));
                }
                odb.Close();

                odb = ODBFactory.Open(file);
                Objects<Function> functions = odb.GetObjects<Function>(new CriteriaQuery(Where.Equal("name","function 199")));
                Console.WriteLine(" Number of functions = " + functions.Count);

                odb.Close();
            }
            catch (Exception e)
            {
                Console.WriteLine(e);
            }
            Console.ReadLine();
        }


        static void Main(string[] args)

        {
            test4();
        }
    }
}
