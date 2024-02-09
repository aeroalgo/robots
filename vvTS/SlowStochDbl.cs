using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200008D RID: 141
	[HandlerCategory("vvStoch"), HandlerName("SlowStoch Dbl")]
	public class SlowStochDbl : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060004E5 RID: 1253 RVA: 0x00019074 File Offset: 0x00017274
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("slowstochDbl", new string[]
			{
				this.Kperiod.ToString(),
				this.D.ToString(),
				src.GetHashCode().ToString()
			}, () => SlowStochDbl.GenSlowStochDbl(src, this.Context, this.Kperiod, this.D));
		}

		// Token: 0x060004E4 RID: 1252 RVA: 0x00018EE8 File Offset: 0x000170E8
		public static IList<double> GenSlowStochDbl(IList<double> _src, IContext _ctx, int kperiod, bool _D)
		{
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				kperiod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.Highest(_src, kperiod));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				kperiod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.Lowest(_src, kperiod));
			IList<double> src = _src;
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				double num = data[i] - data2[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * (src[i] - data2[i]) / num));
			}
			SMA.GenSMA(array, 3);
			IList<double> list = SMA.GenSMA(array, 3);
			IList<double> result = SMA.GenSMA(list, 3);
			if (!_D)
			{
				return list;
			}
			return result;
		}

		// Token: 0x170001AC RID: 428
		public IContext Context
		{
			// Token: 0x060004E6 RID: 1254 RVA: 0x000190F2 File Offset: 0x000172F2
			get;
			// Token: 0x060004E7 RID: 1255 RVA: 0x000190FA File Offset: 0x000172FA
			set;
		}

		// Token: 0x170001AB RID: 427
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool D
		{
			// Token: 0x060004E2 RID: 1250 RVA: 0x00018EA7 File Offset: 0x000170A7
			get;
			// Token: 0x060004E3 RID: 1251 RVA: 0x00018EAF File Offset: 0x000170AF
			set;
		}

		// Token: 0x170001AA RID: 426
		[HandlerParameter(true, "14", Min = "0", Max = "20", Step = "1")]
		public int Kperiod
		{
			// Token: 0x060004E0 RID: 1248 RVA: 0x00018E96 File Offset: 0x00017096
			get;
			// Token: 0x060004E1 RID: 1249 RVA: 0x00018E9E File Offset: 0x0001709E
			set;
		}
	}
}
