using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200018E RID: 398
	[HandlerCategory("vvAverages"), HandlerName("NavelSma")]
	public class NavelSma : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000C90 RID: 3216 RVA: 0x00036660 File Offset: 0x00034860
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("navelsma", new string[]
			{
				this.SmaPeriod.ToString(),
				sec.get_CacheName()
			}, () => NavelSma.GenNavelSMA(sec, this.SmaPeriod));
		}

		// Token: 0x06000C8F RID: 3215 RVA: 0x00036558 File Offset: 0x00034758
		public static IList<double> GenNavelSMA(ISecurity _sec, int maperiod)
		{
			int count = _sec.get_Bars().Count;
			IList<double> closePrices = _sec.get_ClosePrices();
			IList<double> openPrices = _sec.get_OpenPrices();
			IList<double> lowPrices = _sec.get_LowPrices();
			IList<double> highPrices = _sec.get_HighPrices();
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				if (i < maperiod)
				{
					array[i] = closePrices[i];
				}
				else
				{
					double num = 0.0;
					for (int j = 0; j < maperiod; j++)
					{
						double num2 = (closePrices[i - j] * 5.0 + openPrices[i - j] * 2.0 + highPrices[i - j] + lowPrices[i - j]) / 9.0;
						num += num2;
					}
					array[i] = num / (double)maperiod;
				}
			}
			return array;
		}

		// Token: 0x1700041A RID: 1050
		public IContext Context
		{
			// Token: 0x06000C91 RID: 3217 RVA: 0x000366C4 File Offset: 0x000348C4
			get;
			// Token: 0x06000C92 RID: 3218 RVA: 0x000366CC File Offset: 0x000348CC
			set;
		}

		// Token: 0x17000419 RID: 1049
		[HandlerParameter(true, "20", Min = "1", Max = "60", Step = "1")]
		public int SmaPeriod
		{
			// Token: 0x06000C8D RID: 3213 RVA: 0x00036545 File Offset: 0x00034745
			get;
			// Token: 0x06000C8E RID: 3214 RVA: 0x0003654D File Offset: 0x0003474D
			set;
		}
	}
}
