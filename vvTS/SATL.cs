using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200016C RID: 364
	[HandlerCategory("vvAverages"), HandlerName("SATL")]
	public class SATL : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B8C RID: 2956 RVA: 0x0002F943 File Offset: 0x0002DB43
		public IList<double> Execute(IList<double> src)
		{
			return SATL.GenSATL(src);
		}

		// Token: 0x06000B8B RID: 2955 RVA: 0x0002F3A0 File Offset: 0x0002D5A0
		public static IList<double> GenSATL(IList<double> src)
		{
			if (src.Count < 65)
			{
				return null;
			}
			double[] array = new double[src.Count];
			for (int i = 0; i < 65; i++)
			{
				array[i] = src[i];
			}
			for (int j = 65; j < src.Count; j++)
			{
				array[j] = 0.0982862174 * src[j] + 0.0975682269 * src[j - 1] + 0.0961401078 * src[j - 2] + 0.0940230544 * src[j - 3] + 0.091243709 * src[j - 4] + 0.0878391006 * src[j - 5] + 0.0838544303 * src[j - 6] + 0.079340635 * src[j - 7] + 0.0743569346 * src[j - 8] + 0.0689666682 * src[j - 9] + 0.0632381578 * src[j - 10] + 0.0572428925 * src[j - 11] + 0.0510534242 * src[j - 12] + 0.0447468229 * src[j - 13] + 0.038395995 * src[j - 14] + 0.0320735368 * src[j - 15] + 0.0258537721 * src[j - 16] + 0.0198005183 * src[j - 17] + 0.0139807863 * src[j - 18] + 0.0084512448 * src[j - 19] + 0.0032639979 * src[j - 20] - 0.0015350359 * src[j - 21] - 0.0059060082 * src[j - 22] - 0.0098190256 * src[j - 23] - 0.0132507215 * src[j - 24] - 0.0161875265 * src[j - 25] - 0.0186164872 * src[j - 26] - 0.0205446727 * src[j - 27] - 0.0219739146 * src[j - 28] - 0.0229204861 * src[j - 29] - 0.0234080863 * src[j - 30] - 0.0234566315 * src[j - 31] - 0.0231017777 * src[j - 32] - 0.02237969 * src[j - 33] - 0.0213300463 * src[j - 34] - 0.0199924534 * src[j - 35] - 0.0184126992 * src[j - 36] - 0.0166377699 * src[j - 37] - 0.0147139428 * src[j - 38] - 0.0126796776 * src[j - 39] - 0.0105938331 * src[j - 40] - 0.008473677 * src[j - 41] - 0.006384185 * src[j - 42] - 0.0043466731 * src[j - 43] - 0.0023956944 * src[j - 44] - 0.000553518 * src[j - 45] + 0.0011421469 * src[j - 46] + 0.0026845693 * src[j - 47] + 0.0040471369 * src[j - 48] + 0.0052380201 * src[j - 49] + 0.0062194591 * src[j - 50] + 0.0070340085 * src[j - 51] + 0.0076266453 * src[j - 52] + 0.0080376628 * src[j - 53] + 0.0083037666 * src[j - 54] + 0.0083694798 * src[j - 55] + 0.0082901022 * src[j - 56] + 0.0080741359 * src[j - 57] + 0.007754382 * src[j - 58] + 0.0073260526 * src[j - 59] + 0.0068163569 * src[j - 60] + 0.0062325477 * src[j - 61] + 0.0056078229 * src[j - 62] + 0.0049516078 * src[j - 63] + 0.0161380976 * src[j - 64];
			}
			return array;
		}

		// Token: 0x170003CE RID: 974
		public IContext Context
		{
			// Token: 0x06000B8D RID: 2957 RVA: 0x0002F94B File Offset: 0x0002DB4B
			get;
			// Token: 0x06000B8E RID: 2958 RVA: 0x0002F953 File Offset: 0x0002DB53
			set;
		}
	}
}
