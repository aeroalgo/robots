using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200006F RID: 111
	[HandlerCategory("vvIndicators"), HandlerName("Weighted Close")]
	public class WeightedClose : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060003E6 RID: 998 RVA: 0x00015360 File Offset: 0x00013560
		public IList<double> Execute(ISecurity src)
		{
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			double[] array = new double[closePrices.Count];
			for (int i = 0; i < closePrices.Count; i++)
			{
				array[i] = (closePrices[i] * 2.0 + highPrices[i] + lowPrices[i]) / 4.0;
			}
			return array;
		}

		// Token: 0x17000150 RID: 336
		public IContext Context
		{
			// Token: 0x060003E7 RID: 999 RVA: 0x000153D6 File Offset: 0x000135D6
			get;
			// Token: 0x060003E8 RID: 1000 RVA: 0x000153DE File Offset: 0x000135DE
			set;
		}
	}
}
