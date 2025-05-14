using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000043 RID: 67
	[HandlerCategory("vvIndicators"), HandlerName("PCCI (Perfect CCI)")]
	public class PCCI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000265 RID: 613 RVA: 0x0000B5A4 File Offset: 0x000097A4
		public IList<double> Execute(ISecurity sec)
		{
			if (sec.get_Bars().Count < 39)
			{
				return null;
			}
			IList<double> closePrices = sec.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			for (int i = 39; i < closePrices.Count; i++)
			{
				double num = 0.3642399 * closePrices[i] + 0.33441085 * closePrices[i - 1] + 0.25372851 * closePrices[i - 2] + 0.14548806 * closePrices[i - 3] + 0.03934469 * closePrices[i - 4] - 0.03871426 * closePrices[i - 5] - 0.07451349 * closePrices[i - 6] - 0.06903411 * closePrices[i - 7] - 0.03611022 * closePrices[i - 8] + 0.00422528 * closePrices[i - 9] + 0.03382809 * closePrices[i - 10] + 0.04267885 * closePrices[i - 11] + 0.03120441 * closePrices[i - 12] + 0.00816037 * closePrices[i - 13] - 0.01442877 * closePrices[i - 14] - 0.02678947 * closePrices[i - 15] - 0.02525534 * closePrices[i - 16] - 0.0127291 * closePrices[i - 17] + 0.00350063 * closePrices[i - 18] + 0.01565175 * closePrices[i - 19] + 0.01895659 * closePrices[i - 20] + 0.01328613 * closePrices[i - 21] + 0.00252297 * closePrices[i - 22] - 0.00775517 * closePrices[i - 23] - 0.01301467 * closePrices[i - 24] - 0.01164808 * closePrices[i - 25] - 0.00527241 * closePrices[i - 26] + 0.0024875 * closePrices[i - 27] + 0.0079338 * closePrices[i - 28] + 0.00897632 * closePrices[i - 29] + 0.00583939 * closePrices[i - 30] + 0.00059669 * closePrices[i - 31] - 0.00405186 * closePrices[i - 32] - 0.00610944 * closePrices[i - 33] - 0.00509042 * closePrices[i - 34] - 0.00198138 * closePrices[i - 35] + 0.00144873 * closePrices[i - 36] + 0.00373774 * closePrices[i - 37] + 0.01047723 * closePrices[i - 38] - 0.00022625 * closePrices[i - 39];
				array[i] = (sec.get_HighPrices()[i] + sec.get_LowPrices()[i]) / 2.0 - num;
			}
			return array;
		}

		// Token: 0x170000CF RID: 207
		public IContext Context
		{
			// Token: 0x06000266 RID: 614 RVA: 0x0000B95D File Offset: 0x00009B5D
			get;
			// Token: 0x06000267 RID: 615 RVA: 0x0000B965 File Offset: 0x00009B65
			set;
		}
	}
}
