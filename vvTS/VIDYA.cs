using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A5 RID: 421
	[HandlerCategory("vvAverages"), HandlerName("VIDYA")]
	public class VIDYA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D58 RID: 3416 RVA: 0x0003AC24 File Offset: 0x00038E24
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("vidya", new string[]
			{
				this.PeriodCMO.ToString(),
				this.PeriodEma.ToString(),
				src.GetHashCode().ToString()
			}, () => VIDYA.GenVIDYA(src, this.PeriodCMO, this.PeriodEma));
		}

		// Token: 0x06000D57 RID: 3415 RVA: 0x0003AB60 File Offset: 0x00038D60
		public static IList<double> GenVIDYA(IList<double> src, int periodcmo, int periodema)
		{
			double[] array = new double[src.Count];
			IList<double> list = MomentumOsc.GenCMO(src, periodcmo, 0);
			double num = 2.0 / (1.0 + (double)periodema);
			for (int i = 0; i < src.Count; i++)
			{
				if (i < periodema)
				{
					array[i] = src[i];
				}
				else
				{
					double num2 = Math.Abs(list[i] / 100.0);
					array[i] = src[i] * num * num2 + array[i - 1] * (1.0 - num * num2);
				}
			}
			return array;
		}

		// Token: 0x17000456 RID: 1110
		public IContext Context
		{
			// Token: 0x06000D59 RID: 3417 RVA: 0x0003ACA2 File Offset: 0x00038EA2
			get;
			// Token: 0x06000D5A RID: 3418 RVA: 0x0003ACAA File Offset: 0x00038EAA
			set;
		}

		// Token: 0x17000454 RID: 1108
		[HandlerParameter(true, "9", Min = "1", Max = "60", Step = "1")]
		public int PeriodCMO
		{
			// Token: 0x06000D53 RID: 3411 RVA: 0x0003AB3B File Offset: 0x00038D3B
			get;
			// Token: 0x06000D54 RID: 3412 RVA: 0x0003AB43 File Offset: 0x00038D43
			set;
		}

		// Token: 0x17000455 RID: 1109
		[HandlerParameter(true, "30", Min = "1", Max = "60", Step = "1")]
		public int PeriodEma
		{
			// Token: 0x06000D55 RID: 3413 RVA: 0x0003AB4C File Offset: 0x00038D4C
			get;
			// Token: 0x06000D56 RID: 3414 RVA: 0x0003AB54 File Offset: 0x00038D54
			set;
		}
	}
}
