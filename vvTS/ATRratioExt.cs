using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200000F RID: 15
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("ATR Ratio Ext")]
	public class ATRratioExt : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600007E RID: 126 RVA: 0x0000389D File Offset: 0x00001A9D
		public IList<double> Execute(ISecurity src)
		{
			return ATRratioExt.GenATRratio(src, this.ShortAtrPeriod, this.LongAtrPeriod, this.RatioLimit, this.Context);
		}

		// Token: 0x0600007D RID: 125 RVA: 0x00003784 File Offset: 0x00001984
		public static IList<double> GenATRratio(ISecurity src, int shortAtrperiod, int longAtrperiod, double ratioLimit, IContext context)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			IList<double> data = context.GetData("atr", new string[]
			{
				shortAtrperiod.ToString(),
				src.get_CacheName()
			}, () => Series.AverageTrueRange(src.get_Bars(), shortAtrperiod));
			IList<double> data2 = context.GetData("longatr", new string[]
			{
				longAtrperiod.ToString(),
				src.get_CacheName()
			}, () => Series.AverageTrueRange(src.get_Bars(), longAtrperiod));
			for (int i = 0; i < count; i++)
			{
				double num = data[i] / data2[i];
				bool flag = num > ratioLimit;
				array[i] = ((ratioLimit > 0.0) ? ((double)(flag ? 1 : 0)) : num);
			}
			return array;
		}

		// Token: 0x17000029 RID: 41
		public IContext Context
		{
			// Token: 0x0600007F RID: 127 RVA: 0x000038BD File Offset: 0x00001ABD
			get;
			// Token: 0x06000080 RID: 128 RVA: 0x000038C5 File Offset: 0x00001AC5
			set;
		}

		// Token: 0x17000027 RID: 39
		[HandlerParameter(true, "200", Min = "150", Max = "300", Step = "10")]
		public int LongAtrPeriod
		{
			// Token: 0x06000079 RID: 121 RVA: 0x00003729 File Offset: 0x00001929
			get;
			// Token: 0x0600007A RID: 122 RVA: 0x00003731 File Offset: 0x00001931
			set;
		}

		// Token: 0x17000028 RID: 40
		[HandlerParameter(true, "0", Min = "0.75", Max = "1.2", Step = "0.05")]
		public double RatioLimit
		{
			// Token: 0x0600007B RID: 123 RVA: 0x0000373A File Offset: 0x0000193A
			get;
			// Token: 0x0600007C RID: 124 RVA: 0x00003742 File Offset: 0x00001942
			set;
		}

		// Token: 0x17000026 RID: 38
		[HandlerParameter(true, "10", Min = "5", Max = "50", Step = "5")]
		public int ShortAtrPeriod
		{
			// Token: 0x06000077 RID: 119 RVA: 0x00003718 File Offset: 0x00001918
			get;
			// Token: 0x06000078 RID: 120 RVA: 0x00003720 File Offset: 0x00001920
			set;
		}
	}
}
