using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000149 RID: 329
	[HandlerCategory("vvMACD"), HandlerName("MACD (2 MA in)"), InputInfo(1, "Медленная средняя"), InputInfo(0, "Быстрая средняя")]
	public class MACD_2MA : IDoubleAccumHandler, ITwoSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000A1D RID: 2589 RVA: 0x0002A385 File Offset: 0x00028585
		public IList<double> Execute(IList<double> ma1, IList<double> ma2)
		{
			return this.GenMACD2MA(ma1, ma2);
		}

		// Token: 0x06000A1C RID: 2588 RVA: 0x0002A338 File Offset: 0x00028538
		public IList<double> GenMACD2MA(IList<double> _fastma1, IList<double> _slowma2)
		{
			if (_fastma1.Count != _slowma2.Count)
			{
				return null;
			}
			double[] array = new double[_fastma1.Count];
			for (int i = 0; i < _fastma1.Count; i++)
			{
				array[i] = _fastma1[i] - _slowma2[i];
			}
			return array;
		}
	}
}
