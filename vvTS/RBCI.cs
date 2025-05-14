using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200004D RID: 77
	[HandlerCategory("vvIndicators"), HandlerName("RBCI (Range Bound Channel Index)")]
	public class RBCI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060002BB RID: 699 RVA: 0x0000D14C File Offset: 0x0000B34C
		public IList<double> Execute(ISecurity sec)
		{
			if (sec.get_Bars().Count < 55)
			{
				return null;
			}
			IList<double> closePrices = sec.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			for (int i = 55; i < closePrices.Count; i++)
			{
				array[i] = -(-35.52418194 * closePrices[i] - 29.33398965 * closePrices[i - 1] - 18.42774496 * closePrices[i - 2] - 5.341847567 * closePrices[i - 3] + 7.023163695 * closePrices[i - 4] + 16.17628156 * closePrices[i - 5] + 20.65662104 * closePrices[i - 6] + 20.32661158 * closePrices[i - 7] + 16.27023906 * closePrices[i - 8] + 10.35240127 * closePrices[i - 9] + 4.596423992 * closePrices[i - 10] + 0.5817527531 * closePrices[i - 11] - 0.9559211961 * closePrices[i - 12] - 0.2191111431 * closePrices[i - 13] + 1.861734281 * closePrices[i - 14] + 4.04333043 * closePrices[i - 15] + 5.234224328 * closePrices[i - 16] + 4.851086292 * closePrices[i - 17] + 2.960440887 * closePrices[i - 18] + 0.1815496232 * closePrices[i - 19] - 2.591938701 * closePrices[i - 20] - 4.535883446 * closePrices[i - 21] - 5.180855695 * closePrices[i - 22] - 4.54225353 * closePrices[i - 23] - 3.067145982 * closePrices[i - 24] - 1.431012658 * closePrices[i - 25] - 0.2740437883 * closePrices[i - 26] + 0.0260722294 * closePrices[i - 27] - 0.5359717954 * closePrices[i - 28] - 1.62749164 * closePrices[i - 29] - 2.732295856 * closePrices[i - 30] - 3.358959682 * closePrices[i - 31] - 3.221651455 * closePrices[i - 32] - 2.332625794 * closePrices[i - 33] - 0.9760510577 * closePrices[i - 34] + 0.4132650195 * closePrices[i - 35] + 1.420216677 * closePrices[i - 36] + 1.796998735 * closePrices[i - 37] + 1.54127228 * closePrices[i - 38] + 0.8771442423 * closePrices[i - 39] + 0.1561848839 * closePrices[i - 40] - 0.2797065802 * closePrices[i - 41] - 0.2245901578 * closePrices[i - 42] + 0.3278853523 * closePrices[i - 43] + 1.188784148 * closePrices[i - 44] + 2.057741075 * closePrices[i - 45] + 2.627040982 * closePrices[i - 46] + 2.697374234 * closePrices[i - 47] + 2.228994128 * closePrices[i - 48] + 1.353679243 * closePrices[i - 49] + 0.3089253193 * closePrices[i - 50] - 0.6386689841 * closePrices[i - 51] - 1.276670767 * closePrices[i - 52] - 1.513691845 * closePrices[i - 53] - 1.377516078 * closePrices[i - 54] - 1.615617397 * closePrices[i - 55]);
			}
			return array;
		}

		// Token: 0x170000EC RID: 236
		public IContext Context
		{
			// Token: 0x060002BC RID: 700 RVA: 0x0000D626 File Offset: 0x0000B826
			get;
			// Token: 0x060002BD RID: 701 RVA: 0x0000D62E File Offset: 0x0000B82E
			set;
		}
	}
}
