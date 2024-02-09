using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000186 RID: 390
	[HandlerCategory("vvAverages"), HandlerName("MA RSI adaptive")]
	public class MaRsiAdaptive : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C59 RID: 3161 RVA: 0x00035B84 File Offset: 0x00033D84
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("marsiadapt", new string[]
			{
				this.RSIperiod.ToString(),
				this.Speed.ToString(),
				src.GetHashCode().ToString()
			}, () => MaRsiAdaptive.GenMaRsiAdaptive(src, this.RSIperiod, this.Speed));
		}

		// Token: 0x06000C57 RID: 3159 RVA: 0x00035A88 File Offset: 0x00033C88
		public static IList<double> GenMaRsiAdaptive(IList<double> src, int _RSIperiod, double _Speed)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[,] workMaRsi_ = new double[count, 1];
			IList<double> rsi = RSI.GenRSI(src, _RSIperiod, 0, 0, 100, false);
			for (int i = _RSIperiod; i < count; i++)
			{
				array[i] = MaRsiAdaptive.iMaRsi(src, rsi, workMaRsi_, _RSIperiod, _Speed, i, 0);
			}
			return array;
		}

		// Token: 0x06000C58 RID: 3160 RVA: 0x00035ADC File Offset: 0x00033CDC
		private static double iMaRsi(IList<double> src, IList<double> _rsi, double[,] workMaRsi_, int rsiPeriod, double speed, int r, int instanceNo = 0)
		{
			double num = src[r];
			if (r < rsiPeriod)
			{
				workMaRsi_[r, instanceNo] = num;
			}
			else
			{
				workMaRsi_[r, instanceNo] = workMaRsi_[r - 1, instanceNo] + speed * Math.Abs(_rsi[r] / 100.0 - 0.5) * (num - workMaRsi_[r - 1, instanceNo]);
			}
			return workMaRsi_[r, instanceNo];
		}

		// Token: 0x1700040B RID: 1035
		public IContext Context
		{
			// Token: 0x06000C5A RID: 3162 RVA: 0x00035C02 File Offset: 0x00033E02
			get;
			// Token: 0x06000C5B RID: 3163 RVA: 0x00035C0A File Offset: 0x00033E0A
			set;
		}

		// Token: 0x17000409 RID: 1033
		[HandlerParameter(true, "14", Min = "1", Max = "30", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x06000C53 RID: 3155 RVA: 0x00035A63 File Offset: 0x00033C63
			get;
			// Token: 0x06000C54 RID: 3156 RVA: 0x00035A6B File Offset: 0x00033C6B
			set;
		}

		// Token: 0x1700040A RID: 1034
		[HandlerParameter(true, "1.5", Min = "0.1", Max = "5", Step = "0.1")]
		public double Speed
		{
			// Token: 0x06000C55 RID: 3157 RVA: 0x00035A74 File Offset: 0x00033C74
			get;
			// Token: 0x06000C56 RID: 3158 RVA: 0x00035A7C File Offset: 0x00033C7C
			set;
		}
	}
}
