using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000063 RID: 99
	[HandlerCategory("vvIndicators"), HandlerName("Trend Strength Indicator")]
	public class TSIndicator : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600037A RID: 890 RVA: 0x000139B8 File Offset: 0x00011BB8
		public IList<double> Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> data = this.Context.GetData("atr", new string[]
			{
				this.LookBackPeriod.ToString(),
				src.get_CacheName()
			}, () => Series.AverageTrueRange(src.get_Bars(), this.LookBackPeriod));
			IList<double> closePrices = src.get_ClosePrices();
			double[] array = new double[count];
			for (int i = this.LookBackPeriod; i < count; i++)
			{
				array[i] = Math.Abs(closePrices[i] - closePrices[i - this.LookBackPeriod]) / data[i];
			}
			return Series.SMA(Series.SMA(array, 10), 100);
		}

		// Token: 0x1700012A RID: 298
		public IContext Context
		{
			// Token: 0x0600037B RID: 891 RVA: 0x00013A9B File Offset: 0x00011C9B
			get;
			// Token: 0x0600037C RID: 892 RVA: 0x00013AA3 File Offset: 0x00011CA3
			set;
		}

		// Token: 0x17000129 RID: 297
		[HandlerParameter(true, "10", Min = "1", Max = "100", Step = "5")]
		public int LookBackPeriod
		{
			// Token: 0x06000378 RID: 888 RVA: 0x00013982 File Offset: 0x00011B82
			get;
			// Token: 0x06000379 RID: 889 RVA: 0x0001398A File Offset: 0x00011B8A
			set;
		}
	}
}
