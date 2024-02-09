using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200018C RID: 396
	[HandlerCategory("vvAverages"), HandlerName("Modified Optimum Elliptic Filter")]
	public class MOEF : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000C82 RID: 3202 RVA: 0x00036340 File Offset: 0x00034540
		public IList<double> Execute(ISecurity src)
		{
			return this.Context.GetData("MOEF", new string[]
			{
				this.Trigger.ToString(),
				src.GetHashCode().ToString()
			}, () => MOEF.GenMOEF(src, this.Trigger));
		}

		// Token: 0x06000C80 RID: 3200 RVA: 0x000361D8 File Offset: 0x000343D8
		public static IList<double> GenMOEF(ISecurity src, bool trigger)
		{
			double[] array = new double[src.get_Bars().Count];
			double[] array2 = new double[src.get_Bars().Count];
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			for (int i = 1; i < src.get_Bars().Count; i++)
			{
				if (i > 4)
				{
					array[i] = 0.13785 * (2.0 * MOEF.MedPrice(highPrices, lowPrices, i) - MOEF.MedPrice(highPrices, lowPrices, i - 1)) + 0.0007 * (2.0 * MOEF.MedPrice(highPrices, lowPrices, i - 1) - MOEF.MedPrice(highPrices, lowPrices, i - 2)) + 0.13785 * (2.0 * MOEF.MedPrice(highPrices, lowPrices, i - 2) - MOEF.MedPrice(highPrices, lowPrices, i - 3)) + 1.2103 * array[i - 1] - 0.4867 * array[i - 2];
				}
				else
				{
					array[i] = MOEF.MedPrice(highPrices, lowPrices, i);
				}
				array2[i] = array[i - 1];
			}
			if (!trigger)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x06000C81 RID: 3201 RVA: 0x00036303 File Offset: 0x00034503
		private static double MedPrice(IList<double> _highs, IList<double> _lows, int bar)
		{
			return (_highs[bar] + _lows[bar]) / 2.0;
		}

		// Token: 0x17000416 RID: 1046
		public IContext Context
		{
			// Token: 0x06000C83 RID: 3203 RVA: 0x000363AC File Offset: 0x000345AC
			get;
			// Token: 0x06000C84 RID: 3204 RVA: 0x000363B4 File Offset: 0x000345B4
			set;
		}

		// Token: 0x17000415 RID: 1045
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Trigger
		{
			// Token: 0x06000C7E RID: 3198 RVA: 0x000361C5 File Offset: 0x000343C5
			get;
			// Token: 0x06000C7F RID: 3199 RVA: 0x000361CD File Offset: 0x000343CD
			set;
		}
	}
}
