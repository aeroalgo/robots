using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000195 RID: 405
	[HandlerCategory("vvAverages"), HandlerName("RFTL(RSTL)")]
	public class RFTL_RSTL : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000CD6 RID: 3286 RVA: 0x00038838 File Offset: 0x00036A38
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("rftlrstl", new string[]
			{
				this.DrawRSTL.ToString(),
				src.GetHashCode().ToString()
			}, () => RFTL_RSTL.GenRFTLRSTL(src, this.DrawRSTL));
		}

		// Token: 0x06000CD5 RID: 3285 RVA: 0x00037BF4 File Offset: 0x00035DF4
		public static IList<double> GenRFTLRSTL(IList<double> src, bool rstl)
		{
			int num = rstl ? 98 : 44;
			if (src.Count < num)
			{
				return null;
			}
			double[] array = new double[src.Count];
			for (int i = num; i < src.Count; i++)
			{
				if (i < num)
				{
					array[i] = src[i];
				}
				else if (!rstl)
				{
					array[i] = -0.02232324 * src[i] + 0.02268676 * src[i - 1] + 0.08389067 * src[i - 2] + 0.1463038 * src[i - 3] + 0.19282649 * src[i - 4] + 0.21002638 * src[i - 5] + 0.19282649 * src[i - 6] + 0.1463038 * src[i - 7] + 0.08389067 * src[i - 8] + 0.02268676 * src[i - 9] - 0.02232324 * src[i - 10] - 0.04296564 * src[i - 11] - 0.03980614 * src[i - 12] - 0.02082171 * src[i - 13] + 0.00243636 * src[i - 14] + 0.0195058 * src[i - 15] + 0.02460929 * src[i - 16] + 0.01799295 * src[i - 17] + 0.0047054 * src[i - 18] - 0.00831985 * src[i - 19] - 0.01544722 * src[i - 20] - 0.01456262 * src[i - 21] - 0.0073398 * src[i - 22] + 0.00201852 * src[i - 23] + 0.00902504 * src[i - 24] + 0.01093067 * src[i - 25] + 0.00766099 * src[i - 26] + 0.00145478 * src[i - 27] - 0.00447175 * src[i - 28] - 0.00750446 * src[i - 29] - 0.00671646 * src[i - 30] - 0.00304016 * src[i - 31] + 0.00143433 * src[i - 32] + 0.00457475 * src[i - 33] + 0.00517589 * src[i - 34] + 0.00336708 * src[i - 35] + 0.00034406 * src[i - 36] - 0.00233637 * src[i - 37] - 0.0035228 * src[i - 38] - 0.00293522 * src[i - 39] - 0.00114249 * src[i - 40] + 0.00083536 * src[i - 41] + 0.00215524 * src[i - 42] + 0.00604133 * src[i - 43] - 0.00013046 * src[i - 44];
				}
				else
				{
					array[i] = -0.00514293 * src[i] - 0.00398417 * src[i - 1] - 0.00262594 * src[i - 2] - 0.00107121 * src[i - 3] + 0.00066887 * src[i - 4] + 0.00258172 * src[i - 5] + 0.00465269 * src[i - 6] + 0.00686394 * src[i - 7] + 0.00919334 * src[i - 8] + 0.0116172 * src[i - 9] + 0.01411056 * src[i - 10] + 0.01664635 * src[i - 11] + 0.01919533 * src[i - 12] + 0.02172747 * src[i - 13] + 0.0242132 * src[i - 14] + 0.02662203 * src[i - 15] + 0.02892446 * src[i - 16] + 0.03109071 * src[i - 17] + 0.03309496 * src[i - 18] + 0.03490921 * src[i - 19] + 0.03651145 * src[i - 20] + 0.03788045 * src[i - 21] + 0.03899804 * src[i - 22] + 0.03984915 * src[i - 23] + 0.04042329 * src[i - 24] + 0.04071263 * src[i - 25] + 0.04071263 * src[i - 26] + 0.04042329 * src[i - 27] + 0.03984915 * src[i - 28] + 0.03899804 * src[i - 29] + 0.03788045 * src[i - 30] + 0.03651145 * src[i - 31] + 0.03490921 * src[i - 32] + 0.03309496 * src[i - 33] + 0.03109071 * src[i - 34] + 0.02892446 * src[i - 35] + 0.02662203 * src[i - 36] + 0.0242132 * src[i - 37] + 0.02172747 * src[i - 38] + 0.01919533 * src[i - 39] + 0.01664635 * src[i - 40] + 0.01411056 * src[i - 41] + 0.0116172 * src[i - 42] + 0.00919334 * src[i - 43] + 0.00686394 * src[i - 44] + 0.00465269 * src[i - 45] + 0.00258172 * src[i - 46] + 0.00066887 * src[i - 47] - 0.00107121 * src[i - 48] - 0.00262594 * src[i - 49] - 0.00398417 * src[i - 50] - 0.00514293 * src[i - 51] - 0.00609634 * src[i - 52] - 0.00684602 * src[i - 53] - 0.00739452 * src[i - 54] - 0.00774847 * src[i - 55] - 0.0079163 * src[i - 56] - 0.0079094 * src[i - 57] - 0.00774085 * src[i - 58] - 0.00742482 * src[i - 59] - 0.00697718 * src[i - 60] - 0.00641613 * src[i - 61] - 0.00576108 * src[i - 62] - 0.00502957 * src[i - 63] - 0.00423873 * src[i - 64] - 0.00340812 * src[i - 65] - 0.00255923 * src[i - 66] - 0.00170217 * src[i - 67] - 0.00085902 * src[i - 68] - 4.113E-05 * src[i - 69] + 0.000737 * src[i - 70] + 0.00146422 * src[i - 71] + 0.00213007 * src[i - 72] + 0.00272649 * src[i - 73] + 0.00324752 * src[i - 74] + 0.00368922 * src[i - 75] + 0.00405 * src[i - 76] + 0.00433024 * src[i - 77] + 0.00453068 * src[i - 78] + 0.00465046 * src[i - 79] + 0.00469058 * src[i - 80] + 0.00466041 * src[i - 81] + 0.00457855 * src[i - 82] + 0.00442491 * src[i - 83] + 0.00423019 * src[i - 84] + 0.00399201 * src[i - 85] + 0.00372169 * src[i - 86] + 0.00342736 * src[i - 87] + 0.00311822 * src[i - 88] + 0.00280309 * src[i - 89] + 0.00249088 * src[i - 90] + 0.00219089 * src[i - 91] + 0.00191283 * src[i - 92] + 0.00166683 * src[i - 93] + 0.00146419 * src[i - 94] + 0.00131867 * src[i - 95] + 0.00124645 * src[i - 96] + 0.00126836 * src[i - 97] - 0.00401854 * src[i - 98];
				}
			}
			return array;
		}

		// Token: 0x17000432 RID: 1074
		public IContext Context
		{
			// Token: 0x06000CD7 RID: 3287 RVA: 0x000388A4 File Offset: 0x00036AA4
			get;
			// Token: 0x06000CD8 RID: 3288 RVA: 0x000388AC File Offset: 0x00036AAC
			set;
		}

		// Token: 0x17000431 RID: 1073
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool DrawRSTL
		{
			// Token: 0x06000CD3 RID: 3283 RVA: 0x00037BE1 File Offset: 0x00035DE1
			get;
			// Token: 0x06000CD4 RID: 3284 RVA: 0x00037BE9 File Offset: 0x00035DE9
			set;
		}
	}
}
