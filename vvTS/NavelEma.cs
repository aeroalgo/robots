using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200018D RID: 397
	[HandlerCategory("vvAverages"), HandlerName("NavelEma")]
	public class NavelEma : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000C89 RID: 3209 RVA: 0x000364C8 File Offset: 0x000346C8
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("navelema", new string[]
			{
				this.EmaPeriod.ToString(),
				sec.get_CacheName()
			}, () => NavelEma.GenNavelEMA(sec, this.EmaPeriod));
		}

		// Token: 0x06000C88 RID: 3208 RVA: 0x000363D8 File Offset: 0x000345D8
		public static IList<double> GenNavelEMA(ISecurity src, int maperiod)
		{
			int count = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> openPrices = src.get_OpenPrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			double[] array = new double[count];
			double num = 2.0 / (double)(maperiod + 1);
			array[0] = closePrices[0];
			for (int i = 1; i < count; i++)
			{
				double num2 = (closePrices[i] * 5.0 + openPrices[i] * 2.0 + highPrices[i] + lowPrices[i]) / 9.0;
				array[i] = num2 * num + array[i - 1] * (1.0 - num);
			}
			return array;
		}

		// Token: 0x17000418 RID: 1048
		public IContext Context
		{
			// Token: 0x06000C8A RID: 3210 RVA: 0x0003652C File Offset: 0x0003472C
			get;
			// Token: 0x06000C8B RID: 3211 RVA: 0x00036534 File Offset: 0x00034734
			set;
		}

		// Token: 0x17000417 RID: 1047
		[HandlerParameter(true, "20", Min = "1", Max = "60", Step = "1")]
		public int EmaPeriod
		{
			// Token: 0x06000C86 RID: 3206 RVA: 0x000363C5 File Offset: 0x000345C5
			get;
			// Token: 0x06000C87 RID: 3207 RVA: 0x000363CD File Offset: 0x000345CD
			set;
		}
	}
}
