using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200000E RID: 14
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("ATR Ratio")]
	public class ATRratio : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000073 RID: 115 RVA: 0x000036E5 File Offset: 0x000018E5
		public IList<double> Execute(ISecurity src)
		{
			return ATRratio.GenATRratio(src, this.ShortAtrPeriod, this.LongAtrPeriod, this.Context);
		}

		// Token: 0x06000072 RID: 114 RVA: 0x000035F4 File Offset: 0x000017F4
		public static IList<double> GenATRratio(ISecurity src, int shortAtrperiod, int longAtrperiod, IContext context)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			IList<double> data = context.GetData("atr", new string[]
			{
				shortAtrperiod.ToString(),
				src.get_CacheName()
			}, () => Series.AverageTrueRange(src.get_Bars(), shortAtrperiod));
			IList<double> data2 = context.GetData("atr", new string[]
			{
				longAtrperiod.ToString(),
				src.get_CacheName()
			}, () => Series.AverageTrueRange(src.get_Bars(), longAtrperiod));
			for (int i = 0; i < count; i++)
			{
				array[i] = data[i] / data2[i];
			}
			return array;
		}

		// Token: 0x17000025 RID: 37
		public IContext Context
		{
			// Token: 0x06000074 RID: 116 RVA: 0x000036FF File Offset: 0x000018FF
			get;
			// Token: 0x06000075 RID: 117 RVA: 0x00003707 File Offset: 0x00001907
			set;
		}

		// Token: 0x17000024 RID: 36
		[HandlerParameter(true, "200", Min = "150", Max = "300", Step = "10")]
		public int LongAtrPeriod
		{
			// Token: 0x06000070 RID: 112 RVA: 0x000035A8 File Offset: 0x000017A8
			get;
			// Token: 0x06000071 RID: 113 RVA: 0x000035B0 File Offset: 0x000017B0
			set;
		}

		// Token: 0x17000023 RID: 35
		[HandlerParameter(true, "10", Min = "5", Max = "50", Step = "5")]
		public int ShortAtrPeriod
		{
			// Token: 0x0600006E RID: 110 RVA: 0x00003597 File Offset: 0x00001797
			get;
			// Token: 0x0600006F RID: 111 RVA: 0x0000359F File Offset: 0x0000179F
			set;
		}
	}
}
