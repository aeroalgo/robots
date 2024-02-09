using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000038 RID: 56
	[HandlerCategory("vvIndicators"), HandlerName("Momentum")]
	public class Momentum : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000205 RID: 517 RVA: 0x000098F8 File Offset: 0x00007AF8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("momentum", new string[]
			{
				this.MomPeriod.ToString(),
				this.Smooth.ToString(),
				this.SmoothPhase.ToString(),
				src.GetHashCode().ToString()
			}, () => Momentum.GenMomentum(src, this.MomPeriod, this.Smooth, this.SmoothPhase));
		}

		// Token: 0x06000204 RID: 516 RVA: 0x0000986C File Offset: 0x00007A6C
		public static IList<double> GenMomentum(IList<double> src, int _momperiod, int _Smooth, int _SmoothPhase)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				int index = Math.Max(0, i - _momperiod);
				array[i] = src[i] - src[index];
			}
			IList<double> result = array;
			if (_Smooth > 0)
			{
				result = JMA.GenJMA(array, _Smooth, _SmoothPhase);
			}
			return result;
		}

		// Token: 0x170000AF RID: 175
		public IContext Context
		{
			// Token: 0x06000206 RID: 518 RVA: 0x00009988 File Offset: 0x00007B88
			get;
			// Token: 0x06000207 RID: 519 RVA: 0x00009990 File Offset: 0x00007B90
			set;
		}

		// Token: 0x170000AC RID: 172
		[HandlerParameter(true, "10", Min = "2", Max = "20", Step = "1")]
		public int MomPeriod
		{
			// Token: 0x060001FE RID: 510 RVA: 0x00009836 File Offset: 0x00007A36
			get;
			// Token: 0x060001FF RID: 511 RVA: 0x0000983E File Offset: 0x00007A3E
			set;
		}

		// Token: 0x170000AD RID: 173
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000200 RID: 512 RVA: 0x00009847 File Offset: 0x00007A47
			get;
			// Token: 0x06000201 RID: 513 RVA: 0x0000984F File Offset: 0x00007A4F
			set;
		}

		// Token: 0x170000AE RID: 174
		[HandlerParameter(true, "100", Min = "-100", Max = "100", Step = "20")]
		public int SmoothPhase
		{
			// Token: 0x06000202 RID: 514 RVA: 0x00009858 File Offset: 0x00007A58
			get;
			// Token: 0x06000203 RID: 515 RVA: 0x00009860 File Offset: 0x00007A60
			set;
		}
	}
}
