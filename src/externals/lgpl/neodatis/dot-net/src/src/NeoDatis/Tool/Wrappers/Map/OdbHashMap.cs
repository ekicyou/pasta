using System;
using System.Collections.Generic;
using System.Text;

namespace NeoDatis.Tool.Wrappers.Map
{
    public class OdbHashMap3<TKey, TValue> : Dictionary<TKey, TValue>
    {
        public OdbHashMap3()
            : base()
        {
        }

        
        public OdbHashMap3(IDictionary<TKey, TValue> dictionary)
            : base(dictionary)
        {
        }
        public OdbHashMap3(int capacity)
            : base(capacity)
        {
        }
        public virtual bool PutAll(IDictionary<TKey, TValue> map)
        {

            ICollection<TKey> keys = map.Keys;
            foreach (TKey k in keys)
            {
                Add(k, map[k]);
            }
            return true;
        }

        public virtual bool RemoveAll(IDictionary<TKey, TValue> map)
        {
            ICollection<TKey> keys = map.Keys;
            foreach (TKey k in keys)
            {
                Remove(k);
            }
            return true;
        }
        public TValue Remove2(TKey key)
        {

            TValue v = default(TValue);
            TryGetValue(key, out v);
            if (v != null)
            {
                Remove(key);
            }
            return v;
        }
    }
}
