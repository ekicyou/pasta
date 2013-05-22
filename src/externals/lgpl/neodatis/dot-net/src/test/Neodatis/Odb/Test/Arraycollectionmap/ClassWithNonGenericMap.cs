using System;
using System.Text;
using System.Collections;

namespace Test.Neodatis.Odb.Test.Arraycollectionmap
{
    class ClassWithNonGenericMap
    {
        string name;
        IDictionary map;

        public ClassWithNonGenericMap(string name)
        {
            this.name = name;
            map = new Hashtable();
        }
        public void Add(object key, object value)
        {
            map.Add(key, value);
        }
        public object Get(object key)
        {
            return map[key];
        }
        public int Size()
        {
            return map.Count;
        }

        public string GetName()
        {
            return name;
        }
    }
}
