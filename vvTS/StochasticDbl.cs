using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000088 RID: 136
	[HandlerCategory("vvStoch"), HandlerName("Stochastic Dbl")]
	public class StochasticDbl : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060004B7 RID: 1207 RVA: 0x00018144 File Offset: 0x00016344
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("stochasticDbl", new string[]
			{
				this.Kperiod.ToString(),
				this.postSmooth.ToString(),
				this.D.ToString(),
				src.GetHashCode().ToString()
			}, () => StochasticDbl.GenStochasticDbl(src, this.Kperiod, this.postSmooth, this.D, this.Context));
		}

		// Token: 0x060004B6 RID: 1206 RVA: 0x00017FA8 File Offset: 0x000161A8
		public static IList<double> GenStochasticDbl(IList<double> _src, int _kperiod, int _postsmooth, bool _D, IContext _ctx)
		{
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				_kperiod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.Highest(_src, _kperiod));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				_kperiod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.Lowest(_src, _kperiod));
			IList<double> src = _src;
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				double num = data[i] - data2[i];
				array[i] = ((num == 0.0) ? 0.0 : (100.0 * (src[i] - data2[i]) / num));
			}
			IList<double> list = Series.SMA(array, 3);
			if (_postsmooth > 0)
			{
				return JMA.GenJMA(_D ? list : array, _postsmooth, 0);
			}
			if (!_D)
			{
				return array;
			}
			return list;
		}

		// Token: 0x1700019C RID: 412
		public IContext Context
		{
			// Token: 0x060004B8 RID: 1208 RVA: 0x000181D4 File Offset: 0x000163D4
			get;
			// Token: 0x060004B9 RID: 1209 RVA: 0x000181DC File Offset: 0x000163DC
			set;
		}

		// Token: 0x1700019A RID: 410
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool D
		{
			// Token: 0x060004B2 RID: 1202 RVA: 0x00017F57 File Offset: 0x00016157
			get;
			// Token: 0x060004B3 RID: 1203 RVA: 0x00017F5F File Offset: 0x0001615F
			set;
		}

		// Token: 0x17000199 RID: 409
		[HandlerParameter(true, "14", Min = "0", Max = "20", Step = "1")]
		public int Kperiod
		{
			// Token: 0x060004B0 RID: 1200 RVA: 0x00017F46 File Offset: 0x00016146
			get;
			// Token: 0x060004B1 RID: 1201 RVA: 0x00017F4E File Offset: 0x0001614E
			set;
		}

		// Token: 0x1700019B RID: 411
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x060004B4 RID: 1204 RVA: 0x00017F68 File Offset: 0x00016168
			get;
			// Token: 0x060004B5 RID: 1205 RVA: 0x00017F70 File Offset: 0x00016170
			set;
		}
	}
}
