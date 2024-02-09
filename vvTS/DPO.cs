using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000022 RID: 34
	[HandlerCategory("vvIndicators"), HandlerName("Detrended price oscillator")]
	public class DPO : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000134 RID: 308 RVA: 0x00005E44 File Offset: 0x00004044
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> closes = sec.get_ClosePrices();
			IList<double> data = this.Context.GetData("sma", new string[]
			{
				this.Period.ToString(),
				closes.GetHashCode().ToString()
			}, () => Series.SMA(closes, this.Period));
			double[] array = new double[closes.Count];
			for (int i = 0; i < closes.Count; i++)
			{
				array[i] = closes[i] - data[i];
			}
			return JMA.GenJMA(array, this.Smooth, this.SmoothPhase);
		}

		// Token: 0x17000067 RID: 103
		public IContext Context
		{
			// Token: 0x06000135 RID: 309 RVA: 0x00005F16 File Offset: 0x00004116
			get;
			// Token: 0x06000136 RID: 310 RVA: 0x00005F1E File Offset: 0x0000411E
			set;
		}

		// Token: 0x17000064 RID: 100
		[HandlerParameter(true, "14", Min = "1", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x0600012E RID: 302 RVA: 0x00005DF0 File Offset: 0x00003FF0
			get;
			// Token: 0x0600012F RID: 303 RVA: 0x00005DF8 File Offset: 0x00003FF8
			set;
		}

		// Token: 0x17000065 RID: 101
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000130 RID: 304 RVA: 0x00005E01 File Offset: 0x00004001
			get;
			// Token: 0x06000131 RID: 305 RVA: 0x00005E09 File Offset: 0x00004009
			set;
		}

		// Token: 0x17000066 RID: 102
		[HandlerParameter(true, "0", Min = "-100", Max = "100", Step = "25")]
		public int SmoothPhase
		{
			// Token: 0x06000132 RID: 306 RVA: 0x00005E12 File Offset: 0x00004012
			get;
			// Token: 0x06000133 RID: 307 RVA: 0x00005E1A File Offset: 0x0000401A
			set;
		}
	}
}
