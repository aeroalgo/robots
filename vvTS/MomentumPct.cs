using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200003A RID: 58
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("Momentum Pct")]
	public class MomentumPct : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000219 RID: 537 RVA: 0x00009CD8 File Offset: 0x00007ED8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("momentumpct", new string[]
			{
				this.MomPeriod.ToString(),
				this.Smooth.ToString(),
				this.SmoothPhase.ToString(),
				src.GetHashCode().ToString()
			}, () => MomentumPct.GenMomentumPct(src, this.MomPeriod, this.Smooth, this.SmoothPhase));
		}

		// Token: 0x06000218 RID: 536 RVA: 0x00009C34 File Offset: 0x00007E34
		public static IList<double> GenMomentumPct(IList<double> src, int _momperiod, int _Smooth, int _SmoothPhase)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				int index = Math.Max(0, i - _momperiod);
				array[i] = src[i] / Math.Max(1E-05, src[index]) * 100.0;
			}
			IList<double> result = array;
			if (_Smooth > 0)
			{
				result = JMA.GenJMA(array, _Smooth, _SmoothPhase);
			}
			return result;
		}

		// Token: 0x170000B6 RID: 182
		public IContext Context
		{
			// Token: 0x0600021A RID: 538 RVA: 0x00009D68 File Offset: 0x00007F68
			get;
			// Token: 0x0600021B RID: 539 RVA: 0x00009D70 File Offset: 0x00007F70
			set;
		}

		// Token: 0x170000B3 RID: 179
		[HandlerParameter(true, "10", Min = "2", Max = "20", Step = "1")]
		public int MomPeriod
		{
			// Token: 0x06000212 RID: 530 RVA: 0x00009BFF File Offset: 0x00007DFF
			get;
			// Token: 0x06000213 RID: 531 RVA: 0x00009C07 File Offset: 0x00007E07
			set;
		}

		// Token: 0x170000B4 RID: 180
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000214 RID: 532 RVA: 0x00009C10 File Offset: 0x00007E10
			get;
			// Token: 0x06000215 RID: 533 RVA: 0x00009C18 File Offset: 0x00007E18
			set;
		}

		// Token: 0x170000B5 RID: 181
		[HandlerParameter(true, "100", Min = "-100", Max = "100", Step = "20")]
		public int SmoothPhase
		{
			// Token: 0x06000216 RID: 534 RVA: 0x00009C21 File Offset: 0x00007E21
			get;
			// Token: 0x06000217 RID: 535 RVA: 0x00009C29 File Offset: 0x00007E29
			set;
		}
	}
}
