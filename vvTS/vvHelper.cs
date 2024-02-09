using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000002 RID: 2
	public static class vvHelper
	{
		// Token: 0x06000001 RID: 1 RVA: 0x00002050 File Offset: 0x00000250
		public static bool IsNaN(this double d)
		{
			return double.IsNaN(d);
		}

		// Token: 0x06000002 RID: 2 RVA: 0x00002058 File Offset: 0x00000258
		public static bool IsNull(this object obj)
		{
			return obj == null;
		}

		// Token: 0x06000004 RID: 4 RVA: 0x00002098 File Offset: 0x00000298
		public static void LogError(IContext ctx, bool _logging, string str, params object[] args)
		{
			int num = 16711680;
			string text = string.Format("*** Error: " + str, args);
			if (_logging)
			{
				ctx.Log(text, num);
			}
		}

		// Token: 0x06000003 RID: 3 RVA: 0x00002060 File Offset: 0x00000260
		public static void LogInfo(IContext ctx, bool _logging, string str, params object[] args)
		{
			int num = 255;
			string text = string.Format("*** Info: " + str, args);
			if (_logging)
			{
				ctx.Log(text, new Color(num));
			}
		}
	}
}
