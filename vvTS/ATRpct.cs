using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200000B RID: 11
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("ATRpct")]
	public class ATRpct : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000057 RID: 87 RVA: 0x000031D0 File Offset: 0x000013D0
		public IList<double> Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> list = vvSeries.MedianPrice(src.get_Bars());
			double[] array = new double[count];
			IList<double> list2;
			if (this.WATR)
			{
				list2 = ATR.GenWATR(src, this.Period, this.Smooth, this.Context);
			}
			else
			{
				list2 = ATR.GenATR(src, this.Period, this.Smooth);
			}
			for (int i = 0; i < count; i++)
			{
				array[i] = list2[i] / list[i] * 1000.0;
			}
			return array;
		}

		// Token: 0x1700001B RID: 27
		public IContext Context
		{
			// Token: 0x06000058 RID: 88 RVA: 0x00003264 File Offset: 0x00001464
			get;
			// Token: 0x06000059 RID: 89 RVA: 0x0000326C File Offset: 0x0000146C
			set;
		}

		// Token: 0x17000018 RID: 24
		[HandlerParameter(true, "10", Min = "5", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000051 RID: 81 RVA: 0x0000319C File Offset: 0x0000139C
			get;
			// Token: 0x06000052 RID: 82 RVA: 0x000031A4 File Offset: 0x000013A4
			set;
		}

		// Token: 0x17000019 RID: 25
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000053 RID: 83 RVA: 0x000031AD File Offset: 0x000013AD
			get;
			// Token: 0x06000054 RID: 84 RVA: 0x000031B5 File Offset: 0x000013B5
			set;
		}

		// Token: 0x1700001A RID: 26
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool WATR
		{
			// Token: 0x06000055 RID: 85 RVA: 0x000031BE File Offset: 0x000013BE
			get;
			// Token: 0x06000056 RID: 86 RVA: 0x000031C6 File Offset: 0x000013C6
			set;
		}
	}
}
