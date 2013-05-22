
using System;

namespace NeoDatis.Tool.Wrappers{

/**
 * @author olivier
 *
 */
	public class OdbArray {

		public static void SetValue(object array, int index, object value){
			((Array)array).SetValue(value,index);
		}

	}
}