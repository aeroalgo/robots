using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000044 RID: 68
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("Percentage Price Oscillator")]
	public class PPO : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600026E RID: 622 RVA: 0x0000BB74 File Offset: 0x00009D74
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("PercentagePriceOscillator", new string[]
			{
				this.Period1.ToString(),
				this.Period2.ToString(),
				sec.GetHashCode().ToString()
			}, () => PPO.GenPPO(sec, this.Context, this.Period1, this.Period2));
		}

		// Token: 0x0600026D RID: 621 RVA: 0x0000B9C8 File Offset: 0x00009BC8
		public static IList<double> GenPPO(ISecurity sec, IContext _ctx, int _Period1, int _Period2)
		{
			IList<double> C = sec.get_ClosePrices();
			IList<double> list = new List<double>(C.Count);
			IList<double> data = _ctx.GetData("ema", new string[]
			{
				_Period1.ToString(),
				C.GetHashCode().ToString()
			}, () => Series.EMA(C, _Period1));
			IList<double> data2 = _ctx.GetData("ema", new string[]
			{
				_Period2.ToString(),
				C.GetHashCode().ToString()
			}, () => Series.EMA(C, _Period2));
			for (int i = 0; i < C.Count; i++)
			{
				double item;
				if (data2[i] != 0.0)
				{
					item = 100.0 * (data[i] - data2[i]) / data2[i];
				}
				else
				{
					item = 100.0 * (data[i] - data2[i]) / (data2[i] + 1E-07);
				}
				list.Add(item);
			}
			return list;
		}

		// Token: 0x170000D2 RID: 210
		public IContext Context
		{
			// Token: 0x0600026F RID: 623 RVA: 0x0000BBF2 File Offset: 0x00009DF2
			get;
			// Token: 0x06000270 RID: 624 RVA: 0x0000BBFA File Offset: 0x00009DFA
			set;
		}

		// Token: 0x170000D0 RID: 208
		[HandlerParameter(true, "12", Min = "5", Max = "100", Step = "1")]
		public int Period1
		{
			// Token: 0x06000269 RID: 617 RVA: 0x0000B976 File Offset: 0x00009B76
			get;
			// Token: 0x0600026A RID: 618 RVA: 0x0000B97E File Offset: 0x00009B7E
			set;
		}

		// Token: 0x170000D1 RID: 209
		[HandlerParameter(true, "26", Min = "5", Max = "100", Step = "1")]
		public int Period2
		{
			// Token: 0x0600026B RID: 619 RVA: 0x0000B987 File Offset: 0x00009B87
			get;
			// Token: 0x0600026C RID: 620 RVA: 0x0000B98F File Offset: 0x00009B8F
			set;
		}
	}
}
